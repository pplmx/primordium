use crate::model::state::entity::EntityStatus;
use crate::model::state::pheromone::PheromoneType;
use crate::model::state::terrain::TerrainType;
use crate::model::world::World;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

pub struct WebRenderer {
    width: f64,
    height: f64,
    cell_size: f64,
}

impl WebRenderer {
    pub fn new(width: f64, height: f64, world_width: u16, world_height: u16) -> Self {
        Self {
            width,
            height,
            cell_size: width.min(height) / (world_width.max(world_height) as f64),
        }
    }

    pub fn render(&self, ctx: &CanvasRenderingContext2d, world: &World) {
        // Clear background
        ctx.set_fill_style(&JsValue::from_str("#111111"));
        ctx.fill_rect(0.0, 0.0, self.width, self.height);

        let scale_x = self.width / world.width as f64;
        let scale_y = self.height / world.height as f64;

        // Draw Terrain
        for y in 0..world.height {
            for x in 0..world.width {
                let cell = &world.terrain.grid[y as usize][x as usize];
                let color = match cell.terrain_type {
                    TerrainType::Mountain => "#4a4a4a", // Dark Grey
                    TerrainType::River => "#2b5a75",    // Blue
                    TerrainType::Oasis => "#2ecc71",    // Green
                    TerrainType::Plains => "#1a1a1a",   // Black/Dark
                };

                if matches!(cell.terrain_type, TerrainType::Plains) {
                    continue; // Optimize: don't draw plains over background
                }

                ctx.set_fill_style(&JsValue::from_str(color));
                ctx.fill_rect(x as f64 * scale_x, y as f64 * scale_y, scale_x, scale_y);
            }
        }

        // Draw Pheromones (Optional visualization)
        // High density of draw calls, might be slow - skipping for V1 or making very faint

        // Draw Food
        ctx.set_fill_style(&JsValue::from_str("#00ff00"));
        for food in &world.food {
            ctx.begin_path();
            ctx.arc(
                food.x as f64 * scale_x + scale_x / 2.0,
                food.y as f64 * scale_y + scale_y / 2.0,
                scale_x / 2.0,
                0.0,
                std::f64::consts::PI * 2.0,
            )
            .unwrap();
            ctx.fill();
        }

        // Draw Entities
        for entity in &world.entities {
            let status = entity.status(
                world.config.metabolism.reproduction_threshold,
                world.tick,
                world.config.metabolism.maturity_age,
            );
            let color = match status {
                EntityStatus::Starving => "#ff0000", // Red
                EntityStatus::Juvenile => "#cccccc", // Silver
                EntityStatus::Sharing => "#00ff00",  // Green
                EntityStatus::Hunting => "#ff8c00",  // Orange
                EntityStatus::Mating => "#ff69b4",   // Pink
                EntityStatus::Foraging => "#00cc00", // Default Green
            };

            ctx.set_fill_style(&JsValue::from_str(color));

            let ex = entity.physics.x * scale_x;
            let ey = entity.physics.y * scale_y;
            let size = scale_x * 0.8; // Slightly smaller than cell

            ctx.begin_path();
            ctx.arc(ex, ey, size, 0.0, std::f64::consts::PI * 2.0)
                .unwrap();
            ctx.fill();

            // Draw territorial range or interaction if needed? No, too cluttered.
        }
    }
}
