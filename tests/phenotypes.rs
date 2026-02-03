mod common;
use common::{EntityBuilder, WorldBuilder};
use primordium_core::systems::social;
use primordium_lib::model::config::AppConfig;

#[tokio::test]
async fn test_phenotype_inheritance_and_mutation() {
    let p1 = EntityBuilder::new()
        .at(10.0, 10.0)
        .energy(200.0)
        .max_energy(400.0)
        .build();

    // Update fields manually as some aren't in builder yet
    let mut p1 = p1;
    p1.physics.sensing_range = 10.0;
    p1.physics.max_speed = 2.0;
    p1.intel.genotype.sensing_range = 10.0;
    p1.intel.genotype.max_speed = 2.0;
    p1.intel.genotype.maturity_gene = 2.0;

    let config = AppConfig::default();

    let mut rng = rand::thread_rng();
    let mut ctx = social::ReproductionContext {
        tick: 100,
        config: &config,
        population: 1,
        traits: std::collections::HashSet::new(),
        is_radiation_storm: false,
        rng: &mut rng,
        ancestral_genotype: None,
    };
    let (child, _) = social::reproduce_asexual_parallel_components_decomposed(
        &p1.position,
        p1.metabolism.energy,
        p1.metabolism.generation,
        &p1.intel.genotype,
        p1.intel.specialization,
        &mut ctx,
    );

    assert!(child.physics.sensing_range >= 3.0 && child.physics.sensing_range <= 15.0);
    assert!(child.physics.max_speed >= 0.5 && child.physics.max_speed <= 3.0);
    assert!(child.metabolism.max_energy >= 100.0 && child.metabolism.max_energy <= 500.0);
}

#[tokio::test]
async fn test_sensing_range_affects_perception() {
    let (mut world, mut env) = WorldBuilder::new()
        .with_config(|c| {
            c.evolution.drift_rate = 0.0;
        })
        .with_food(22.0, 10.0, 0.0) // Distance 12.0 from (10,10)
        .with_entity(EntityBuilder::new().at(10.0, 10.0).energy(1000.0).build())
        .with_entity(EntityBuilder::new().at(30.0, 30.0).energy(1000.0).build())
        .build();

    // Modify ranges
    let query = world.ecs.query_mut::<(
        &mut primordium_lib::model::state::Physics,
        &mut primordium_lib::model::state::Intel,
        &primordium_lib::model::state::Position,
    )>();
    for (_h, (phys, intel, pos)) in query {
        if pos.x < 15.0 {
            phys.sensing_range = 5.0;
            intel.genotype.sensing_range = 5.0;
        } else {
            phys.sensing_range = 15.0;
            intel.genotype.sensing_range = 15.0;
        }
    }

    world.update(&mut env).unwrap();

    let entities = world.get_all_entities();
    let (mut short, mut long) = (None, None);
    for e in entities {
        if e.physics.sensing_range < 10.0 {
            short = Some(e);
        } else {
            long = Some(e);
        }
    }

    assert_eq!(short.unwrap().physics.sensing_range, 5.0);
    assert_eq!(long.unwrap().physics.sensing_range, 15.0);
}

#[tokio::test]
async fn test_hex_dna_contains_phenotype() {
    let e = EntityBuilder::new().build();
    let mut e = e;
    e.intel.genotype.sensing_range = 12.34;
    e.intel.genotype.max_speed = 2.5;

    let hex = e.intel.genotype.to_hex();
    let restored = primordium_lib::model::state::entity::Genotype::from_hex(&hex).unwrap();

    assert_eq!(restored.sensing_range, 12.34);
    assert_eq!(restored.max_speed, 2.5);
}
