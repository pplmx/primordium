use primordium_core::systems::action::{action_system, ActionContext, ActionOutput};
use primordium_lib::model::config::AppConfig;
use primordium_lib::model::state::environment::Environment;
use primordium_lib::model::state::terrain::TerrainGrid;
use primordium_lib::model::world::InternalEntitySnapshot;

#[tokio::test]
async fn test_symbiosis_spring_force() {
    let mut e1 = primordium_lib::model::lifecycle::create_entity(10.0, 10.0, 0);
    let e2 = primordium_lib::model::lifecycle::create_entity(15.0, 10.0, 0); // 5.0 units away (Rest length is 2.0)

    e1.intel.bonded_to = Some(e2.identity.id);

    let snapshot = InternalEntitySnapshot {
        id: e2.identity.id,
        lineage_id: e2.metabolism.lineage_id,
        x: e2.position.x,
        y: e2.position.y,
        energy: 100.0,
        birth_tick: 0,
        offspring_count: 0,
        generation: e2.metabolism.generation,
        max_energy: e2.metabolism.max_energy,
        r: 0,
        g: 0,
        b: 0,
        rank: 0.5,
        status: primordium_lib::model::state::entity::EntityStatus::Bonded,
        genotype: Some(e2.intel.genotype.clone()),
    };

    let env = Environment::default();
    let config = AppConfig::default();
    let mut terrain = TerrainGrid::generate(20, 20, 42);
    terrain.set_cell_type(
        10,
        10,
        primordium_lib::model::state::terrain::TerrainType::Plains,
    );
    terrain.set_cell_type(
        11,
        10,
        primordium_lib::model::state::terrain::TerrainType::Plains,
    );

    let mut id_map = std::collections::HashMap::new();
    id_map.insert(e2.identity.id, 0);

    let influence = primordium_lib::model::influence::InfluenceGrid::new(100, 100);
    let mut ctx = ActionContext {
        env: &env,
        config: &config,
        terrain: &terrain,
        influence: &influence,
        snapshots: &[snapshot],
        entity_id_map: &id_map,
        spatial_hash: &primordium_lib::model::spatial_hash::SpatialHash::new(5.0, 100, 100),
        pressure: &primordium_lib::model::pressure::PressureGrid::new(100, 100),
        width: 100,
        height: 100,
    };

    // Outputs: Neutral movement (should stay still if no spring)
    // outputs[0] (dx) = 0.0 -> target vx 0.0
    let outputs = [0.0; 12];

    e1.velocity.vx = 0.0;
    e1.velocity.vy = 0.0;

    {
        let mut out = ActionOutput::default();
        action_system(&mut e1, outputs, &mut ctx, &mut out);
        out
    };

    // Spring should pull e1 towards e2 (positive x direction)
    // e1 is at 10, e2 is at 15. Force vector is (1, 0).
    assert!(
        e1.velocity.vx > 0.001,
        "Spring force should pull entity towards partner. VX: {}",
        e1.velocity.vx
    );
}
