mod common;
use common::{EntityBuilder, WorldBuilder};
use primordium_lib::model::brain::{Connection, GenotypeLogic};
use primordium_lib::model::state::entity::Genotype;
use std::sync::Arc;

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
    for i in 0..10 {
        let e = EntityBuilder::new()
            .at(80.0 + i as f64, 80.0)
            .lineage(ancestor_id)
            .energy(200.0)
            .build();
        world_builder = world_builder.with_entity(e);
    }

    let (mut world, mut env) = world_builder.build();

    // Run for 2000 ticks to allow divergence
    for _ in 0..2000 {
        for (_h, met) in world
            .ecs
            .query_mut::<&mut primordium_lib::model::state::Metabolism>()
        {
            met.energy = 200.0;
        }
        world.update(&mut env).expect("Update failed");

        if world.get_population_count() > 300 {
            break;
        }
    }

    // Collect genotypes
    let mut left_genomes: Vec<Arc<Genotype>> = Vec::new();
    let mut right_genomes: Vec<Arc<Genotype>> = Vec::new();

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

    assert!(!left_genomes.is_empty());
    assert!(!right_genomes.is_empty());

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

    assert!(avg_dist > 0.5);
}

#[tokio::test]
async fn test_reproductive_isolation_emergence() {
    let world_builder = WorldBuilder::new().with_seed(42).with_config(|c| {
        c.evolution.speciation_threshold = 2.0;
    });

    let id_a = uuid::Uuid::new_v4();
    let id_b = uuid::Uuid::new_v4();

    let mut e1 = EntityBuilder::new()
        .at(10.0, 10.0)
        .energy(500.0)
        .lineage(id_a)
        .build();
    {
        let brain = &mut Arc::make_mut(&mut e1.intel.genotype).brain;
        brain.connections = vec![Connection {
            from: 0,
            to: 29,
            weight: 5.0,
            enabled: true,
            innovation: 1,
        }];
        use primordium_lib::model::brain::BrainLogic;
        brain.initialize_node_idx_map();
    }

    let mut e2 = EntityBuilder::new()
        .at(10.0, 10.0)
        .energy(500.0)
        .lineage(id_b)
        .build();
    {
        let brain = &mut Arc::make_mut(&mut e2.intel.genotype).brain;
        brain.connections = vec![Connection {
            from: 0,
            to: 29,
            weight: -5.0,
            enabled: true,
            innovation: 1,
        }];
        use primordium_lib::model::brain::BrainLogic;
        brain.initialize_node_idx_map();
    }

    let dist = e1.intel.genotype.distance(&e2.intel.genotype);
    assert!(dist > 1.0);

    let (_world, _env) = world_builder.build();
}
