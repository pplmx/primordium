use primordium_lib::model::brain::Connection;
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::entity::{Entity, EntityStatus};
use primordium_lib::model::state::environment::Environment;

use primordium_lib::model::world::World;

#[test]
fn test_rank_accumulation() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();

    let mut e = Entity::new(10.0, 10.0, 0);
    e.metabolism.energy = e.metabolism.max_energy; // Max energy score (0.3)
    e.metabolism.offspring_count = 20; // Max offspring score (0.1)
    e.metabolism.birth_tick = 0;
    e.intel.reputation = 1.0; // Max rep score (0.3)

    // Total should be roughly 0.3 + 0.1 + 0.3 + AgeScore
    // At tick 0, age is 0.

    world.entities.push(e);

    // Run update to trigger Pass 0 rank calc
    world.update(&mut env).expect("Update failed");

    let rank = world.entities[0].intel.rank;
    // Energy(1.0)*0.3 + Age(0)*0.3 + Offspring(1.0)*0.1 + Rep(1.0)*0.3 = 0.7
    assert!(
        rank >= 0.69,
        "Rank calculation incorrect, expected ~0.7, got {}",
        rank
    );

    // Now simulate aging
    world.tick = 2000;
    world.update(&mut env).expect("Update failed");
    let rank_aged = world.entities[0].intel.rank;
    // Age(2000)*0.3 = 0.3. Total ~1.0
    assert!(rank_aged > rank, "Rank should increase with age");
}

#[test]
fn test_soldier_classification() {
    let mut e = Entity::new(0.0, 0.0, 0);
    e.intel.rank = 0.9;
    e.intel.last_aggression = 0.8;

    // Tick=200 > Maturity=100 -> Mature
    let status = e.status(100.0, 200, 100);
    assert_eq!(
        status,
        EntityStatus::Soldier,
        "High rank + Aggression should be Soldier"
    );

    e.intel.last_aggression = 0.1;
    let status2 = e.status(100.0, 0, 100);
    assert_ne!(
        status2,
        EntityStatus::Soldier,
        "Low aggression should not be Soldier"
    );
}

#[test]
fn test_tribal_split_under_pressure() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();

    // Create a crowded scenario
    // 20 entities in a small area
    for _ in 0..20 {
        let mut e = Entity::new(10.0, 10.0, 0);
        e.physics.r = 100; // Original Tribe
        e.intel.reputation = 0.0;
        e.metabolism.energy = 10.0;
        world.entities.push(e);
    }

    // Run update
    world.update(&mut env).expect("Update failed");

    // Check if any entity changed color
    let _original_color = (100, 100, 100); // Wait, Entity::new randomizes color?
                                           // Ah, update sets r,g,b.
                                           // Let's check if we have DIVERSE colors now.

    let mut distinct_colors = std::collections::HashSet::new();
    for e in &world.entities {
        distinct_colors.insert((e.physics.r, e.physics.g, e.physics.b));
    }

    assert!(
        distinct_colors.len() > 5,
        "Tribal split should generate new colors in crowded low-rank population"
    );
}

#[test]
fn test_soldier_damage_bonus() {
    // This requires inspecting `handle_predation` logic indirectly or via unit test in social.rs.
    // But since logic is embedded in World::update InteractionCommand generation, integration test is best.
    // However, it's hard to precisely measure damage in integration without mocking rng.
    // But we can check if a powerful soldier kills a strong target.

    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.metabolism.reproduction_threshold = 10000.0; // Disable reproduction
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();

    let mut soldier = Entity::new(10.0, 10.0, 0);
    world.tick = 1000; // Ensure mature

    // FORCE AGGRESSION VIA BRAIN
    soldier.intel.genotype.brain.connections.clear();
    // Connect Input 2 (Energy) to Output 23 (Aggro) with high weight
    soldier.intel.genotype.brain.connections.push(Connection {
        from: 2,
        to: 23,
        weight: 10.0,
        enabled: true,
        innovation: 9999,
    });

    soldier.intel.rank = 0.9;
    soldier.metabolism.offspring_count = 20; // Boost calculated rank
    soldier.intel.last_aggression = 0.9; // Soldier
    soldier.metabolism.energy = 200.0;
    soldier.metabolism.trophic_potential = 1.0; // Hunter

    let mut victim = Entity::new(10.0, 10.0, 0);
    victim.metabolism.energy = 200.0; // Strong victim
                                      // Different tribe
    soldier.physics.r = 255;
    victim.physics.r = 0;

    // Verify status manually
    let status = soldier.status(10000.0, 1000, 150);
    println!("Soldier Status: {:?}", status);

    world.entities.push(soldier);
    world.entities.push(victim);

    // 1 tick might be enough if Soldier bonus applies (1.5x)
    // Attacker Power = 200 * 1.5 = 300.
    // Victim Resistance = 200 / 0.4 (def_mult) = 500? Wait.
    // Default defense_mult is 1.0 if no allies?
    // In social.rs: defense_mult = (1.0 - allies*0.15).max(0.4).
    // If no allies, defense_mult = 1.0.
    // So Victim Res = 200.
    // Attacker Power = 200 (base) * 1.5 (Soldier) = 300.
    // 300 > 200 -> Kill.

    world.update(&mut env).expect("Update failed");

    assert_eq!(
        world.entities.len(),
        1,
        "Soldier should have killed the victim due to damage bonus"
    );
}
