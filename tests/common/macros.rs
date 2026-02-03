/// Asserts that an entity with the given ID has at least the specified amount of energy.
#[macro_export]
macro_rules! assert_energy_above {
    ($world:expr, $id:expr, $min_energy:expr) => {
        let entity = $world
            .get_all_entities()
            .into_iter()
            .find(|e| e.identity.id == $id)
            .expect("Entity not found in world");
        assert!(
            entity.metabolism.energy > $min_energy,
            "Entity {} energy {} is not above {}",
            $id,
            entity.metabolism.energy,
            $min_energy
        );
    };
}

/// Asserts that an entity with the given ID is NOT present in the world (dead/despawned).
#[macro_export]
macro_rules! assert_entity_dead {
    ($world:expr, $id:expr) => {
        let exists = $world
            .get_all_entities()
            .into_iter()
            .any(|e| e.identity.id == $id);
        assert!(!exists, "Entity {} should be dead but was found alive", $id);
    };
}

/// Asserts that the total population count matches the expected value.
#[macro_export]
macro_rules! assert_population {
    ($world:expr, $count:expr) => {
        assert_eq!(
            $world.get_population_count(),
            $count,
            "Population count mismatch"
        );
    };
}
