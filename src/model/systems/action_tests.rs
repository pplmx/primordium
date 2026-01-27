#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::state::environment::Environment;
    #[test]
    fn test_action_system_energy_consumption() {
        let mut entity = Entity::new(5.0, 5.0, 0);
        entity.metabolism.energy = 100.0;
        let initial_energy = entity.metabolism.energy;
        let outputs = [0.0; 12];
        let env = Environment::default();
        let config = AppConfig::default();
        let terrain = TerrainGrid::generate(20, 20, 42);
        let pressure_grid = crate::model::state::pressure::PressureGrid::new(20, 20);
        let mut ctx = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            snapshots: &[],
            entity_id_map: &std::collections::HashMap::new(),
            spatial_hash: &crate::model::quadtree::SpatialHash::new(5.0, 20, 20),
            pressure: &pressure_grid,
            width: 20,
            height: 20,
        };
        action_system(&mut entity, outputs, &mut ctx);
        assert!(entity.metabolism.energy < initial_energy);
    }
    #[test]
    fn test_action_system_predation_mode_higher_cost() {
        let mut entity_normal = Entity::new(5.0, 5.0, 0);
        let mut entity_predator = Entity::new(5.0, 5.0, 0);
        entity_normal.metabolism.energy = 100.0;
        entity_predator.metabolism.energy = 100.0;
        let normal_outputs = [0.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let predator_outputs = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let env = Environment::default();
        let config = AppConfig::default();
        let terrain = TerrainGrid::generate(20, 20, 42);
        let pressure_grid = crate::model::state::pressure::PressureGrid::new(20, 20);
        let mut ctx_n = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            snapshots: &[],
            entity_id_map: &std::collections::HashMap::new(),
            spatial_hash: &crate::model::quadtree::SpatialHash::new(5.0, 20, 20),
            pressure: &pressure_grid,
            width: 20,
            height: 20,
        };
        action_system(&mut entity_normal, normal_outputs, &mut ctx_n);
        let mut ctx_p = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            snapshots: &[],
            entity_id_map: &std::collections::HashMap::new(),
            spatial_hash: &crate::model::quadtree::SpatialHash::new(5.0, 20, 20),
            pressure: &pressure_grid,
            width: 20,
            height: 20,
        };
        action_system(&mut entity_predator, predator_outputs, &mut ctx_p);
        assert!(entity_predator.metabolism.energy < entity_normal.metabolism.energy);
    }
    #[test]
    fn test_action_system_velocity_update() {
        let mut entity = Entity::new(5.0, 5.0, 0);
        entity.physics.vx = 0.0;
        entity.physics.vy = 0.0;
        let outputs = [1.0, -1.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let env = Environment::default();
        let config = AppConfig::default();
        let mut terrain = TerrainGrid::generate(20, 20, 42);
        terrain.set_cell_type(5, 5, crate::model::state::terrain::TerrainType::Plains);
        let pressure_grid = crate::model::state::pressure::PressureGrid::new(20, 20);
        let mut ctx = ActionContext {
            env: &env,
            config: &config,
            terrain: &terrain,
            snapshots: &[],
            entity_id_map: &std::collections::HashMap::new(),
            spatial_hash: &crate::model::quadtree::SpatialHash::new(5.0, 20, 20),
            pressure: &pressure_grid,
            width: 20,
            height: 20,
        };
        action_system(&mut entity, outputs, &mut ctx);
        assert!(entity.physics.vx > 0.0);
        assert!(entity.physics.vy < 0.0);
    }
}
