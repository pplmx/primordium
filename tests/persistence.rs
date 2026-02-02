use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::world::World;

#[tokio::test] async
fn test_world_persistence_cycle() {
    let config = AppConfig::default();
    let mut env = Environment::default();
    let mut world = World::new(10, config).expect("Failed to create world");

    world.update(&mut env).expect("Failed to update world");

    println!("Testing Entity serialization...");
    let entities = world.get_all_entities();
    if let Some(e) = entities.first() {
        let _ = serde_json::to_string(e).expect("Failed to serialize Entity");
        println!("Entity OK");
    }

    println!("Testing Food serialization...");
    let food_list: Vec<_> = world
        .ecs
        .query::<&primordium_lib::model::state::Food>()
        .iter()
        .map(|(_, f)| f.clone())
        .collect();
    if let Some(f) = food_list.first() {
        let _ = serde_json::to_string(f).expect("Failed to serialize Food");
        println!("Food OK");
    }

    println!("Testing Terrain serialization...");
    let _ = serde_json::to_string(&world.terrain).expect("Failed to serialize Terrain");
    println!("Terrain OK");

    println!("Testing Pheromones serialization...");
    let _ = serde_json::to_string(&world.pheromones).expect("Failed to serialize Pheromones");
    println!("Pheromones OK");

    println!("Starting full World serialization...");
    let serialized = serde_json::to_string(&world).expect("Failed to serialize World");
    println!("World OK, size: {}", serialized.len());

    let _deserialized: World =
        serde_json::from_str(&serialized).expect("Failed to deserialize World");
    println!("Deserialization OK");
}
