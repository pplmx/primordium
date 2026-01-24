use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::{Block, Borders, Widget};

use crate::model::state::terrain::TerrainType;
use crate::model::world::World;

pub struct WorldWidget<'a> {
    world: &'a World,
    screensaver: bool,
    view_mode: u8, // NEW
}

impl<'a> WorldWidget<'a> {
    pub fn new(world: &'a World, screensaver: bool, view_mode: u8) -> Self {
        Self {
            world,
            screensaver,
            view_mode,
        }
    }

    pub fn get_inner_area(area: Rect, screensaver: bool) -> Rect {
        if screensaver {
            area
        } else {
            Block::default().borders(Borders::ALL).inner(area)
        }
    }

    pub fn world_to_screen(
        world_x: f64,
        world_y: f64,
        area: Rect,
        screensaver: bool,
    ) -> Option<(u16, u16)> {
        let inner = Self::get_inner_area(area, screensaver);
        let x = inner.x + world_x as u16;
        let y = inner.y + world_y as u16;
        if x >= inner.left() && x < inner.right() && y >= inner.top() && y < inner.bottom() {
            Some((x, y))
        } else {
            None
        }
    }

    pub fn screen_to_world(
        screen_x: u16,
        screen_y: u16,
        area: Rect,
        screensaver: bool,
    ) -> Option<(f64, f64)> {
        let inner = Self::get_inner_area(area, screensaver);
        if screen_x >= inner.left()
            && screen_x < inner.right()
            && screen_y >= inner.top()
            && screen_y < inner.bottom()
        {
            Some(((screen_x - inner.x) as f64, (screen_y - inner.y) as f64))
        } else {
            None
        }
    }
}

impl<'a> Widget for WorldWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.screensaver {
            let block = Block::default()
                .title(format!("World (Tick: {})", self.world.tick))
                .borders(Borders::ALL);
            block.render(area, buf);
        }

        let inner = Self::get_inner_area(area, self.screensaver);

        // 0. Render Terrain Layer (background)
        for y in 0..inner.height.min(self.world.terrain.height) {
            for x in 0..inner.width.min(self.world.terrain.width) {
                let terrain = self.world.terrain.get_cell(x, y);
                let screen_x = inner.x + x;
                let screen_y = inner.y + y;
                if screen_x < inner.right() && screen_y < inner.bottom() {
                    let cell = buf.get_mut(screen_x, screen_y);

                    // VIEW MODES
                    match self.view_mode {
                        1 => {
                            // FERTILITY HEATMAP
                            let f = terrain.fertility;
                            let r = (255.0 * (1.0 - f)) as u8;
                            let g = (255.0 * f) as u8;
                            cell.set_bg(Color::Rgb(r / 4, g / 2, 0));
                            if terrain.terrain_type != TerrainType::Plains {
                                cell.set_symbol(terrain.terrain_type.symbol().to_string().as_str());
                                cell.set_fg(terrain.terrain_type.color());
                            }
                        }
                        2 => {
                            // SOCIAL ZONES & REPUTATION HEATMAP
                            let sm = self.world.social_grid[y as usize][x as usize];
                            if sm == 1 {
                                cell.set_bg(Color::Rgb(0, 0, 100)); // Peace
                            } else if sm == 2 {
                                cell.set_bg(Color::Rgb(100, 0, 0)); // War
                            } else {
                                cell.set_bg(Color::Rgb(20, 20, 20));
                            }
                            if terrain.terrain_type != TerrainType::Plains {
                                cell.set_symbol(terrain.terrain_type.symbol().to_string().as_str());
                                cell.set_fg(terrain.terrain_type.color());
                            }
                        }
                        3 => {
                            // RANK HEATMAP
                            // Find highest rank in world for normalization
                            let max_rank = self
                                .world
                                .entities
                                .iter()
                                .map(|e| e.intel.rank)
                                .fold(0.0, f32::max)
                                .max(0.1);

                            // Find average rank at this cell
                            let mut cell_rank = 0.0;
                            let mut count = 0;
                            let radius = 3.0;
                            let nearby = self.world.spatial_hash.query(x as f64, y as f64, radius);
                            for &idx in &nearby {
                                cell_rank += self.world.entities[idx].intel.rank;
                                count += 1;
                            }
                            if count > 0 {
                                let val = (cell_rank / count as f32) / max_rank;
                                let intensity = (val * 255.0) as u8;
                                cell.set_bg(Color::Rgb(intensity / 2, 0, intensity));
                            // Purple/Magenta for rank
                            } else {
                                cell.set_bg(Color::Rgb(10, 10, 10));
                            }

                            if terrain.terrain_type != TerrainType::Plains {
                                cell.set_symbol(terrain.terrain_type.symbol().to_string().as_str());
                                cell.set_fg(terrain.terrain_type.color());
                            }
                        }
                        4 => {
                            // VOCAL PROPAGATION HEATMAP
                            // Sense vocalization at this cell
                            let mut vocal_sum = 0.0;
                            let radius = 5.0;
                            let nearby = self.world.spatial_hash.query(x as f64, y as f64, radius);
                            for &idx in &nearby {
                                let dist = ((self.world.entities[idx].physics.x - x as f64)
                                    .powi(2)
                                    + (self.world.entities[idx].physics.y - y as f64).powi(2))
                                .sqrt();
                                let falloff = (1.0 - (dist / radius)).max(0.0) as f32;
                                vocal_sum +=
                                    self.world.entities[idx].intel.last_vocalization * falloff;
                            }

                            let intensity = (vocal_sum.min(1.0) * 255.0) as u8;
                            if intensity > 10 {
                                cell.set_bg(Color::Rgb(intensity, intensity, 0));
                            // Yellow for sound
                            } else {
                                cell.set_bg(Color::Rgb(10, 10, 10));
                            }

                            if terrain.terrain_type != TerrainType::Plains {
                                cell.set_symbol(terrain.terrain_type.symbol().to_string().as_str());
                                cell.set_fg(terrain.terrain_type.color());
                            }
                        }
                        _ => {
                            // NORMAL VIEW
                            if terrain.terrain_type != TerrainType::Plains {
                                cell.set_symbol(terrain.terrain_type.symbol().to_string().as_str());
                                cell.set_fg(terrain.terrain_type.color());
                                if terrain.fertility < 0.3
                                    && terrain.terrain_type != TerrainType::Desert
                                {
                                    cell.set_fg(Color::Rgb(100, 50, 50));
                                }
                            }
                            // Show Social Zones even in normal view but subtle
                            let sm = self.world.social_grid[y as usize][x as usize];
                            if sm == 1 {
                                cell.set_bg(Color::Rgb(0, 0, 40));
                            } else if sm == 2 {
                                cell.set_bg(Color::Rgb(40, 0, 0));
                            }
                        }
                    }
                }
            }
        }

        // 1. Render Food (Green '*')
        for food in &self.world.food {
            if let Some((x, y)) =
                Self::world_to_screen(f64::from(food.x), f64::from(food.y), area, self.screensaver)
            {
                let cell = buf.get_mut(x, y);
                cell.set_symbol(&food.symbol.to_string());
                cell.set_fg(ratatui::style::Color::Rgb(
                    food.color_rgb.0,
                    food.color_rgb.1,
                    food.color_rgb.2,
                ));
            }
        }

        // 2. Optimized Entity Rendering
        for entity in &self.world.entities {
            if let Some((x, y)) =
                Self::world_to_screen(entity.physics.x, entity.physics.y, area, self.screensaver)
            {
                let status = entity.status(
                    self.world.config.metabolism.reproduction_threshold,
                    self.world.tick,
                    self.world.config.metabolism.maturity_age,
                );
                let cell = buf.get_mut(x, y);
                cell.set_symbol(&entity.symbol_for_status(status).to_string());
                cell.set_fg(entity.color_for_status(status));

                // Alpha/Soldier Aura in certain view modes
                if self.view_mode == 3 || self.view_mode == 2 {
                    if entity.intel.rank > 0.9 {
                        // Alpha Aura - Pulsing? No, just highlight
                        cell.set_bg(Color::Rgb(100, 100, 0));
                    } else if status == crate::model::state::entity::EntityStatus::Soldier {
                        // Soldier Aura
                        cell.set_bg(Color::Rgb(80, 0, 0));
                    }
                }
            }
        }
    }
}
