mod common;
use common::{EntityBuilder, WorldBuilder};
use primordium_core::systems::civilization;
use primordium_lib::model::terrain::{OutpostSpecialization, TerrainType};
use uuid::Uuid;

#[tokio::test]
async fn test_contested_ownership() {
    let lineage_a = Uuid::new_v4();
    let lineage_b = Uuid::new_v4();

    let mut world_builder = WorldBuilder::new()
        .with_outpost(25, 25, lineage_a) // Sets energy to 500.0, we want 100.0
        .with_entity(
            EntityBuilder::new()
                .at(25.0, 25.0)
                .energy(30.0)
                .lineage(lineage_a)
                .build(),
        );

    for _ in 0..3 {
        world_builder = world_builder.with_entity(
            EntityBuilder::new()
                .at(26.0, 26.0)
                .energy(100.0)
                .lineage(lineage_b)
                .build(),
        );
    }

    let (mut world, _env) = world_builder.build();

    // Manually override outpost energy to match original test condition
    let idx = world.terrain.index(25, 25);
    world.terrain.cells[idx].energy_store = 100.0;

    world.prepare_spatial_hash();
    world.capture_entity_snapshots();

    civilization::resolve_contested_ownership(
        &mut world.terrain,
        world.width,
        world.height,
        &world.spatial_hash,
        &world.entity_snapshots,
        &world.lineage_registry,
    );

    assert_eq!(
        world.terrain.cells[idx].owner_id,
        Some(lineage_b),
        "Ownership should transfer to lineage_b"
    );
    assert!(
        world.terrain.cells[idx].energy_store < 100.0,
        "Energy store should be reduced during transition"
    );
    assert_eq!(
        world.terrain.cells[idx].outpost_spec,
        OutpostSpecialization::Standard,
        "Specialization should reset to Standard"
    );
}

#[tokio::test]
async fn test_outpost_upgrades() {
    let l_id = Uuid::new_v4();
    
    // Test Case 1: Healthy Tribe -> Silo
    {
        let mut world_builder = WorldBuilder::new()
            .with_outpost(10, 10, l_id);
            
        // We need to set civ level to 2. WorldBuilder doesn't have this yet,
        // but we can assume level 0 is fine if the logic just checks counts?
        // Ah, logic checks `record.civilization_level >= 2`.
        // We need to inject this state.
        
        for _ in 0..3 {
            world_builder = world_builder.with_entity(
                EntityBuilder::new()
                    .at(11.0, 11.0)
                    .lineage(l_id)
                    .energy(80.0) // Healthy
                    .build()
            );
        }
        
        let (mut world, _env) = world_builder.build();
        
        // Manual setup for civ level (Builder gap)
        world.lineage_registry.record_birth(l_id, 0, 0);
        if let Some(record) = world.lineage_registry.lineages.get_mut(&l_id) {
            record.civilization_level = 2;
        }

        world.prepare_spatial_hash();
        world.capture_entity_snapshots();

        civilization::resolve_outpost_upgrades(
            &mut world.terrain,
            world.width,
            world.height,
            &world.spatial_hash,
            &world.entity_snapshots,
            &world.lineage_registry,
        );

        let idx = world.terrain.index(10, 10);
        assert_eq!(
            world.terrain.cells[idx].outpost_spec,
            OutpostSpecialization::Silo,
            "Should upgrade to Silo for healthy tribe"
        );
    }

    // Test Case 2: Weak Tribe -> Nursery
    {
        let mut world_builder = WorldBuilder::new()
            .with_outpost(10, 10, l_id);
            
        for _ in 0..3 {
            world_builder = world_builder.with_entity(
                EntityBuilder::new()
                    .at(11.0, 11.0)
                    .lineage(l_id)
                    .energy(20.0) // Weak
                    .build()
            );
        }
        
        let (mut world, _env) = world_builder.build();
        
        // Manual setup for civ level
        world.lineage_registry.record_birth(l_id, 0, 0);
        if let Some(record) = world.lineage_registry.lineages.get_mut(&l_id) {
            record.civilization_level = 2;
        }

        world.prepare_spatial_hash();
        world.capture_entity_snapshots();

        civilization::resolve_outpost_upgrades(
            &mut world.terrain,
            world.width,
            world.height,
            &world.spatial_hash,
            &world.entity_snapshots,
            &world.lineage_registry,
        );

        let idx = world.terrain.index(10, 10);
        assert_eq!(
            world.terrain.cells[idx].outpost_spec,
            OutpostSpecialization::Nursery,
            "Should upgrade to Nursery for weak tribe"
        );
    }
}

#[tokio::test]
async fn test_dark_age_collapse_and_recovery() {
    let l_id = Uuid::new_v4();
    let idx_10_10 = 10 + 10 * 50; // Manual index calc for assertion (width=50 default)

    // Phase 1: Golden Age
    // Establish a thriving outpost
    let mut world_builder = WorldBuilder::new()
        .with_outpost(10, 10, l_id)
        .with_memory(l_id, "knowledge", 1.0); // High collective memory

    // Add Alpha/Entities
    for i in 0..5 {
        world_builder = world_builder.with_entity(
            EntityBuilder::new()
                .at(10.0 + i as f64, 10.0)
                .lineage(l_id)
                .energy(500.0)
                .build()
        );
    }

    let (mut world, mut env) = world_builder.build();
    let idx = world.terrain.index(10, 10);
    
    // Verify initial state
    assert!(world.terrain.cells[idx].energy_store > 0.0);
    assert_eq!(world.lineage_registry.get_memory_value(&l_id, "knowledge"), 1.0);

    // Phase 2: The Cataclysm (Kill everyone)
    // We can simulate this by manually clearing the ECS
    world.ecs.clear(); 
    // And decaying memory
    
    // Phase 3: Dark Age (Decay)
    for _ in 0..100 {
        // Run update logic that handles decay (environment update)
        world.lineage_registry.decay_memory(0.95);
        
        // Simulate outpost decay (energy loss due to no maintenance)
        // Actual game logic for outpost decay is in handle_outposts_ecs, 
        // which reduces energy if no owner is nearby.
        // We need to run the full update loop or call the system.
        // Since ECS is empty, update() is safe.
        world.update(&mut env).expect("Update failed during Dark Age");
    }

    // Verify Knowledge Lost
    let knowledge = world.lineage_registry.get_memory_value(&l_id, "knowledge");
    assert!(knowledge < 0.1, "Collective memory did not decay enough (Val: {})", knowledge);
    
    // Verify Outpost Abandoned/Decayed
    let cell = &world.terrain.cells[idx];
    // If logic works, energy should drain or ownership lost
    // Outpost ownership persists until claimed by another, but energy drains.
    // Let's check energy.
    assert!(cell.energy_store < 500.0, "Outpost energy did not decay without maintenance");

    // Phase 4: Recovery (New settlers)
    let mut new_settler = EntityBuilder::new()
        .at(10.0, 10.0)
        .lineage(l_id)
        .energy(100.0)
        .build();
    world.spawn_entity(new_settler);

    for _ in 0..50 {
        world.update(&mut env).expect("Recovery update failed");
    }

    // Verify recovery started (energy store increasing or stable, memory might not recover instantly)
    let cell_recovered = &world.terrain.cells[idx];
    assert!(cell_recovered.energy_store > 0.0, "Outpost should be active again");
}
