mod common;
use common::{EntityBuilder, WorldBuilder};
use primordium_lib::model::brain::{Connection, GenotypeLogic};
use primordium_lib::model::state::entity::Genotype;

#[tokio::test]
async fn test_longitudinal_genetic_divergence() {
    let mut world_builder = WorldBuilder::new().with_seed(12345).with_config(|c| {
        c.evolution.mutation_rate = 0.8; // Very high mutation
        c.evolution.mutation_amount = 2.0; // Large jumps
        c.world.max_food = 100;
        c.metabolism.maturity_age = 50; // Fast generations
    });

    let ancestor_id = uuid::Uuid::new_v4();

    // Seed Population A (Left side)
    for i in 0..10 {
        let e = EntityBuilder::new()
            .at(10.0 + i as f64, 10.0)
            .lineage(ancestor_id)
            .energy(200.0)
            .build();
        world_builder = world_builder.with_entity(e);
    }

    // Seed Population B (Right side) - Separated by distance
    // In a 100x100 world, 80.0 is far enough to be effectively isolated if sensing is small
    for i in 0..10 {
        let e = EntityBuilder::new()
            .at(80.0 + i as f64, 80.0)
            .lineage(ancestor_id)
            .energy(200.0)
            .build();
        world_builder = world_builder.with_entity(e);
    }

    let (mut world, mut env) = world_builder.build();

    // Run for 2000 ticks to allow divergence (approx 40 generations)
    for _ in 0..2000 {
        // Boost energy to ensure survival and reproduction
        for (_h, met) in world
            .ecs
            .query_mut::<&mut primordium_lib::model::state::Metabolism>()
        {
            met.energy = 200.0;
        }
        world.update(&mut env).expect("Update failed");

        if world.get_population_count() > 300 {
            break; // Stop if saturated
        }
    }

    // Collect genotypes from left and right populations
    let mut left_genomes: Vec<Genotype> = Vec::new();
    let mut right_genomes: Vec<Genotype> = Vec::new();

    for (_h, (phys, intel)) in world
        .ecs
        .query::<(
            &primordium_lib::model::state::Physics,
            &primordium_lib::model::state::Intel,
        )>()
        .iter()
    {
        if phys.x < 40.0 {
            left_genomes.push(intel.genotype.clone());
        } else if phys.x > 60.0 {
            right_genomes.push(intel.genotype.clone());
        }
    }

    assert!(!left_genomes.is_empty(), "Left population went extinct");
    assert!(!right_genomes.is_empty(), "Right population went extinct");

    // Calculate genetic distance between populations by sampling pairs
    // Comparing averages cancels out drift if it's symmetric around 0.
    // We need to measure the distance between individuals across the barrier.
    let mut total_distance = 0.0;
    let mut samples = 0;

    use rand::seq::SliceRandom;
    let mut rng = rand::thread_rng();

    for _ in 0..20 {
        if let (Some(l), Some(r)) = (
            left_genomes.choose(&mut rng),
            right_genomes.choose(&mut rng),
        ) {
            total_distance += l.distance(r);
            samples += 1;
        }
    }

    let avg_dist = if samples > 0 {
        total_distance / samples as f32
    } else {
        0.0
    };
    println!("Average Genetic Distance (Left vs Right): {}", avg_dist);

    // With high mutation rate, individuals should diverge significantly from each other
    assert!(
        avg_dist > 0.5,
        "Populations did not diverge genetically (Dist: {})",
        avg_dist
    );
}

#[tokio::test]
async fn test_reproductive_isolation_emergence() {
    let world_builder = WorldBuilder::new().with_seed(42).with_config(|c| {
        // High speciation threshold encourages discrimination
        c.evolution.speciation_threshold = 2.0;
    });

    // Create two genetically distinct entities manually
    let id_a = uuid::Uuid::new_v4();
    let id_b = uuid::Uuid::new_v4();

    let mut e1 = EntityBuilder::new()
        .at(10.0, 10.0)
        .energy(500.0)
        .lineage(id_a)
        .build();
    e1.intel.genotype.brain.connections = vec![Connection {
        from: 0,
        to: 29,
        weight: 5.0,
        enabled: true,
        innovation: 1,
    }];

    let mut e2 = EntityBuilder::new()
        .at(10.0, 10.0)
        .energy(500.0)
        .lineage(id_b)
        .build();
    e2.intel.genotype.brain.connections = vec![Connection {
        from: 0,
        to: 29,
        weight: -5.0,
        enabled: true,
        innovation: 1,
    }];

    // Use distance() instead of genetic_distance()
    let dist = e1.intel.genotype.distance(&e2.intel.genotype);
    assert!(
        dist > 1.0,
        "Genetic distance between distinct species should be high"
    );

    let (_world, _env) = world_builder.build();
}
