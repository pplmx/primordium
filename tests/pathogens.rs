use primordium_data::Pathogen;
use primordium_lib::model::config::{AppConfig, GameMode};
use primordium_lib::model::lifecycle;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[test]
fn test_pathogen_transmission() {
    let log_dir = "logs_test_pathogens";
    let _ = std::fs::remove_dir_all(log_dir);
    let mut config = AppConfig::default();
    config.world.initial_population = 0;
    config.game_mode = GameMode::Cooperative; // Disable predation
    let mut world = World::new_at(0, config, log_dir).expect("Failed to create world");
    let mut env = Environment::default();

    // 1. Setup Infected Patient Zero
    let mut patient_zero = lifecycle::create_entity(10.0, 10.0, 0);
    patient_zero.velocity.vy = 0.0;
    let pathogen = Pathogen {
        id: uuid::Uuid::new_v4(),
        lethality: 0.1,
        transmission: 1.0, // Guaranteed spread
        duration: 100,
        virulence: 2.0, // High virulence
        behavior_manipulation: None,
    };
    patient_zero.health.pathogen = Some(pathogen.clone());
    patient_zero.health.infection_timer = pathogen.duration;
    world.spawn_entity(patient_zero);

    // 2. Setup Victim nearby (same position to be sure)
    let mut victim = lifecycle::create_entity(10.0, 10.0, 0);
    victim.velocity.vx = 0.0;
    victim.velocity.vy = 0.0;
    victim.health.immunity = 0.0; // Ensure no immunity for deterministic test
    world.spawn_entity(victim);

    // 3. Update world to spread infection
    world.update(&mut env).expect("Update failed");

    // 4. Verify victim is infected
    let entities = world.get_all_entities();
    for (i, e) in entities.iter().enumerate() {
        println!("Entity {}: Infected={}", i, e.health.pathogen.is_some());
    }

    let infected_count = entities
        .iter()
        .filter(|e| e.health.pathogen.is_some())
        .count();
    assert_eq!(
        infected_count, 2,
        "Pathogen should have spread to neighbor in 1 tick"
    );
}

#[test]
fn test_immunity_gain() {
    let mut entity = lifecycle::create_entity(0.0, 0.0, 0);
    let initial_immunity = entity.health.immunity;

    let pathogen = Pathogen {
        id: uuid::Uuid::new_v4(),
        lethality: 0.0,
        transmission: 1.0,
        duration: 1, // Rapid recovery
        virulence: 2.0,
        behavior_manipulation: None,
    };

    entity.health.pathogen = Some(pathogen);
    entity.health.infection_timer = 1;

    use primordium_lib::model::systems::biological;
    biological::process_infection_components(&mut entity.health, &mut entity.metabolism); // timer -> 0
    biological::process_infection_components(&mut entity.health, &mut entity.metabolism); // recovered

    assert!(entity.health.pathogen.is_none());
    assert!(
        entity.health.immunity > initial_immunity,
        "Entity should gain immunity after recovery"
    );
}
