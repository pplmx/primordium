use primordium_data::EntityStatus;
use primordium_lib::model::brain::Connection;
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::lifecycle;
use primordium_lib::model::state::environment::Environment;

use primordium_lib::model::world::World;

#[tokio::test]
#[ignore]
async fn test_rank_accumulation() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.metabolism.reproduction_threshold = 10000.0; // Disable reproduction
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();

    let mut e = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    e.metabolism.has_metamorphosed = true;
    e.metabolism.energy = e.metabolism.max_energy; // Max energy score (0.3)
    e.metabolism.offspring_count = 20; // Max offspring score (0.1)
    e.metabolism.birth_tick = 0;
    e.intel.reputation = 1.0; // Max rep score (0.3)

    // Total should be roughly 0.3 + 0.1 + 0.3 + AgeScore
    // At tick 0, age is 0.

    world.spawn_entity(e);

    // Run update to trigger Pass 0 rank calc
    world.update(&mut env).expect("Update failed");

    let entities = world.get_all_entities();
    let rank = entities[0].intel.rank;
    // Energy(1.0)*0.3 + Age(0)*0.3 + Offspring(1.0)*0.1 + Rep(1.0)*0.3 = 0.7
    assert!(
        rank >= 0.69,
        "Rank calculation incorrect, expected ~0.7, got {}",
        rank
    );

    // Past peak age (age_rank_normalization is 2000.0, peak is 1400)
    world.tick = 1500;
    world.update(&mut env).expect("Update failed");
    let entities_aged = world.get_all_entities();
    let rank_aged = entities_aged[0].intel.rank;

    let age = 600u64 - entities_aged[0].metabolism.birth_tick;
    let energy_score =
        (entities_aged[0].metabolism.energy / entities_aged[0].metabolism.max_energy) as f32;
    let offspring_score = entities_aged[0].metabolism.offspring_count as f32 / 500.0;
    let rep_score = entities_aged[0].intel.reputation;

    eprintln!(
        "Age: {}, Energy: {:.3}, Offspring: {:.3}, Rep: {:.3}",
        age, energy_score, offspring_score, rep_score
    );
    eprintln!("Initial rank: {:.3}, Aged rank: {:.3}", rank, rank_aged);

    // Past peak age - rank should be lower than initial rank due to bell-curve decay
    assert!(
        rank_aged < rank,
        "Rank should decrease after peak age due to bell-curve decay"
    );
}

#[tokio::test]
async fn test_soldier_classification() {
    let mut e = primordium_lib::model::lifecycle::create_entity(0.0, 0.0, 0);
    e.intel.rank = 0.9;
    e.intel.last_aggression = 0.8;
    e.metabolism.has_metamorphosed = true;

    // Tick=200 > Maturity=100 -> Mature
    let status = lifecycle::calculate_status(&e.metabolism, &e.health, &e.intel, 0.5, 200, 100);
    assert_eq!(
        status,
        EntityStatus::Soldier,
        "High rank + Aggression should be Soldier"
    );

    e.intel.last_aggression = 0.1;
    let status2 = lifecycle::calculate_status(&e.metabolism, &e.health, &e.intel, 0.5, 0, 100);
    assert_ne!(
        status2,
        EntityStatus::Soldier,
        "Low aggression should not be Soldier"
    );
}

#[tokio::test]
async fn test_tribal_split_under_pressure() {
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();

    let lid = uuid::Uuid::new_v4();
    world.lineage_registry.record_birth(lid, 0, 0);

    for i in 0..20 {
        let mut e = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
        e.physics.r = 100;
        e.intel.reputation = 1.0;
        e.metabolism.energy = 500.0;
        e.metabolism.max_energy = 500.0;
        e.metabolism.lineage_id = lid;
        std::sync::Arc::make_mut(&mut e.intel.genotype).lineage_id = lid;
        e.intel.last_aggression = 0.5;
        e.metabolism.offspring_count = i as u32;
        world.spawn_entity(e);
    }

    // Run update
    world.update(&mut env).expect("Update failed");

    let mut distinct_colors = std::collections::HashSet::new();
    for e in world.get_all_entities() {
        distinct_colors.insert((e.physics.r, e.physics.g, e.physics.b));
    }

    assert!(
        distinct_colors.len() > 1,
        "Tribal split should generate new colors with alpha-led migration under crowding"
    );
}

#[tokio::test]
async fn test_soldier_damage_bonus() {
    // This requires inspecting `handle_predation` logic indirectly or via unit test in social.rs.
    // But since logic is embedded in World::update InteractionCommand generation, integration test is best.
    // However, it's hard to precisely measure damage in integration without mocking rng.
    // But we can check if a powerful soldier kills a strong target.

    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.metabolism.reproduction_threshold = 10000.0; // Disable reproduction
    let mut world = World::new(0, config).expect("Failed to create world");
    let mut env = Environment::default();

    let mut soldier = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    soldier.metabolism.has_metamorphosed = true;
    world.tick = 1000; // Ensure mature

    // FORCE AGGRESSION VIA BRAIN
    {
        let brain = &mut std::sync::Arc::make_mut(&mut soldier.intel.genotype).brain;
        brain.connections.clear();
        brain.connections.push(Connection {
            from: 2,
            to: 32, // Aggro
            weight: 10.0,
            enabled: true,
            innovation: 9999,
        });
        use primordium_lib::model::brain::BrainLogic;
        brain.initialize_node_idx_map();
    }

    soldier.intel.rank = 0.9;
    soldier.metabolism.offspring_count = 20; // Boost calculated rank
    soldier.intel.last_aggression = 0.9; // Soldier
    soldier.metabolism.energy = 200.0;
    soldier.metabolism.max_energy = 200.0;
    soldier.metabolism.trophic_potential = 1.0; // Hunter

    let mut victim = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    victim.metabolism.energy = 200.0; // Strong victim
    victim.metabolism.max_energy = 200.0;
    // Ensure victim doesn't attack back
    {
        let brain = &mut std::sync::Arc::make_mut(&mut victim.intel.genotype).brain;
        brain.connections.clear();
        use primordium_lib::model::brain::BrainLogic;
        brain.initialize_node_idx_map();
    }
    // Different tribe
    soldier.physics.r = 255;
    victim.physics.r = 0;

    // Verify status manually
    let status = lifecycle::calculate_status(
        &soldier.metabolism,
        &soldier.health,
        &soldier.intel,
        0.5,
        1000,
        150,
    );
    println!("Soldier Status: {:?}", status);

    world.spawn_entity(soldier);
    world.spawn_entity(victim);

    // 1 tick might be enough if Soldier bonus applies (1.5x)
    // ... logic explained in comments ...

    world.update(&mut env).expect("Update failed");

    assert_eq!(
        world.get_population_count(),
        1,
        "Soldier should have killed the victim due to damage bonus"
    );
}
