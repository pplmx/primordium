mod common;
use common::{EntityBuilder, TestBehavior, WorldBuilder};

#[tokio::test]
async fn test_tribe_solidarity_no_aggression() {
    let id = uuid::Uuid::new_v4();

    // Use builder but explicitly handle the shared lineage ID requirement
    let e1 = EntityBuilder::new()
        .at(10.0, 10.0)
        .energy(5000.0)
        .max_energy(10000.0)
        .color(100, 100, 100)
        .lineage(id)
        .build();

    let mut e1_mut = e1.clone();
    e1_mut.metabolism.trophic_potential = 1.0;
    
    let e2 = EntityBuilder::new()
        .at(10.5, 10.5)
        .energy(5000.0)
        .max_energy(10000.0)
        .color(100, 100, 100)
        .lineage(id)
        .build();
    
    let mut e2_mut = e2.clone();
    e2_mut.metabolism.trophic_potential = 0.0;

    // IMPORTANT: Ensure lineage_id is propagated to both metabolism AND genotype
    // The builder handles this if .lineage() is called, but let's double check via manual override
    // to be absolutely sure the test logic holds.
    // Actually, EntityBuilder::lineage() sets both. The issue might be e2 being recognized as "food" or "prey"
    // despite being kin if the aggression logic overrides kinship for different trophic levels.
    // However, the test asserts NO aggression.
    
    // Let's force them to be identical except for trophic potential
    e2_mut.metabolism.lineage_id = id;
    e2_mut.intel.genotype.lineage_id = id;

    let (mut world, mut env) = WorldBuilder::new()
        .with_entity(e1_mut)
        .with_entity(e2_mut)
        .build();

    for _ in 0..50 {
        world.update(&mut env).expect("Update failed");
    }
    assert!(
        world.get_population_count() >= 2,
        "Hunter attacked its own tribe!"
    );
}

#[tokio::test]
async fn test_energy_sharing_between_allies() {
    let world_builder = WorldBuilder::new();
    
    // Giver
    let e1 = EntityBuilder::new()
        .at(10.0, 10.0)
        .energy(800.0)
        .max_energy(1000.0)
        .color(200, 200, 200)
        .with_behavior(TestBehavior::Altruist)
        .build();
        
    // Receiver
    let e2 = EntityBuilder::new()
        .at(10.2, 10.2)
        .energy(10.0)
        .max_energy(1000.0)
        .color(200, 200, 200)
        .build(); // No special behavior needed
        
    // Clone genotype to ensure compatibility if needed, though color is primary trigger
    let mut e2_clone = e2.clone();
    e2_clone.intel.genotype = e1.intel.genotype.clone();
    
    let (mut world, mut env) = world_builder
        .with_entity(e1)
        .with_entity(e2_clone)
        .build();
        
    let e2_id = world.get_all_entities()[1].identity.id; // Assuming order preserved or we find by energy

    let mut shared = false;
    for _ in 0..100 {
        world.update(&mut env).expect("Update failed");
        let entities = world.get_all_entities();
        if let Some(e2_curr) = entities.iter().find(|e| e.identity.id == e2_id) {
            if e2_curr.metabolism.energy > 15.0 {
                shared = true;
                break;
            }
        }
        
        // Keep E1 energy high and force share intent (manual override still useful for deterministic forcing)
        for (_handle, (phys, met, intel, ident)) in world.ecs.query_mut::<(
            &mut primordium_lib::model::state::Physics,
            &mut primordium_lib::model::state::Metabolism,
            &mut primordium_lib::model::state::Intel,
            &primordium_lib::model::state::Identity,
        )>() {
            if phys.r == 200 && ident.id != e2_id {
                met.energy = 800.0;
                intel.last_share_intent = 1.0;
            }
        }
    }
    assert!(shared, "Energy sharing did not occur between allies");
}

#[tokio::test]
async fn test_inter_tribe_predation() {
    let e1 = EntityBuilder::new()
        .at(10.0, 10.0)
        .color(255, 0, 0)
        .energy(5000.0)
        .with_behavior(TestBehavior::Aggressive)
        .build();
    // Trophic potential needs manual set as builder defaults to 0.5 usually? 
    // Actually builder just builds default entity. We need to set it.
    let mut e1_mut = e1.clone();
    e1_mut.metabolism.trophic_potential = 1.0;

    let e2 = EntityBuilder::new()
        .at(10.1, 10.1)
        .color(0, 0, 255) // Different color = different tribe
        .energy(10.0)
        .build();
    let mut e2_mut = e2.clone();
    e2_mut.metabolism.trophic_potential = 0.0;

    let (mut world, mut env) = WorldBuilder::new()
        .with_entity(e1_mut)
        .with_entity(e2_mut)
        .build();

    for _ in 0..200 {
        world.update(&mut env).expect("Update failed");
        if world.get_population_count() == 1 {
            break;
        }
    }
    // E2 should be eaten
    assert_eq!(world.get_population_count(), 1, "Predator failed to eat prey");
}
