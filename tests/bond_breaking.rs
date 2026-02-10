use primordium_lib::model::config::AppConfig;
use primordium_lib::model::lifecycle;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[tokio::test]
async fn test_bond_break_on_low_reciprocity() {
    let mut config = AppConfig::default();
    config.social.sharing_threshold = 0.5;
    config.world.initial_population = 0;

    let mut world = World::new(0, config.clone()).unwrap();
    let mut env = Environment::default();

    // 1. Create pair with bonding gene
    let mut e1 = lifecycle::create_entity(10.0, 10.0, 0);
    let mut e2 = lifecycle::create_entity(10.5, 10.5, 0);

    e1.metabolism.energy = 500.0;
    e2.metabolism.energy = 50.0; // Needs energy

    // Force bond
    e1.intel.bonded_to = Some(e2.identity.id);
    e2.intel.bonded_to = Some(e1.identity.id);

    // E1 is selfish - high threshold for sharing but bonded
    // Set output[8] (Share) to low value manually to simulate brain decision
    e1.intel.last_activations.0[33] = 0.1; // Output 33 is Share/Bond (index 8 of outputs)

    // E1 brain config to ensure output 8 is low
    // We manually override brain output for test determinism
    // Or we mutate brain to produce low output 8

    // Let's modify the brain to have a bias connection to node 37 (Bond)
    // Node 37 is Bond output. Node 33 is Share.
    // Bond output > 0.5 keeps bond. < 0.2 breaks it.

    // Clear connections and add inhibitory bias to Bond output (37)
    {
        let brain = &mut std::sync::Arc::make_mut(&mut e1.intel.genotype).brain;
        brain.connections.clear();

        // Add bias node (we don't have one, but Input 2 (Energy) is high)
        // Energy (2) -> Bond (37). Weight -5.0
        brain
            .connections
            .push(primordium_lib::model::brain::Connection {
                from: 2,
                to: 37,
                weight: -5.0,
                enabled: true,
                innovation: 1,
            });

        use primordium_lib::model::brain::BrainLogic;
        brain.initialize_node_idx_map();
    }

    world.spawn_entity(e1);
    world.spawn_entity(e2);

    // Update world
    world.update(&mut env).unwrap();

    // Check if bond broken
    let entities = world.get_all_entities();
    // Find E1 by energy
    if let Some(e1_new) = entities.iter().find(|e| e.metabolism.energy > 300.0) {
        assert!(
            e1_new.intel.bonded_to.is_none(),
            "Bond should be broken due to low bond output"
        );
    } else {
        panic!("E1 not found");
    }
}
