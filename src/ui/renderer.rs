use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::{Block, Borders, Widget};
use std::collections::HashMap;

use crate::model::state::terrain::TerrainType;
use crate::model::world::World;

pub struct WorldWidget<'a> {
    world: &'a World,
    screensaver: bool,
    view_mode: u8,
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

        // Pre-calculate screen positions for O(1) partner lookup
        let mut screen_positions = HashMap::new();
        for entity in &self.world.entities {
            if let Some((x, y)) =
                Self::world_to_screen(entity.physics.x, entity.physics.y, area, self.screensaver)
            {
                screen_positions.insert(entity.id, (x, y));
            }
        }

        for y in 0..inner.height.min(self.world.terrain.height) {
            for x in 0..inner.width.min(self.world.terrain.width) {
                let terrain = self.world.terrain.get_cell(x, y);
                let screen_x = inner.x + x;
                let screen_y = inner.y + y;
                if screen_x < inner.right() && screen_y < inner.bottom() {
                    let cell = buf.get_mut(screen_x, screen_y);

                    match self.view_mode {
                        1 => {
                            // FERTILITY
                            let f = terrain.fertility;
                            cell.set_bg(Color::Rgb(
                                (255.0 * (1.0 - f)) as u8 / 4,
                                (255.0 * f) as u8 / 2,
                                0,
                            ));
                        }
                        2 => {
                            // SOCIAL
                            let sm = self.world.social_grid[y as usize][x as usize];
                            if sm == 1 {
                                cell.set_bg(Color::Rgb(0, 0, 100));
                            } else if sm == 2 {
                                cell.set_bg(Color::Rgb(100, 0, 0));
                            } else {
                                cell.set_bg(Color::Rgb(20, 20, 20));
                            }
                        }
                        3 => {
                            // RANK
                            let max_rank = self
                                .world
                                .entities
                                .iter()
                                .map(|e| e.intel.rank)
                                .fold(0.0, f32::max)
                                .max(0.1);
                            let mut cell_rank = 0.0;
                            let nearby = self.world.spatial_hash.query(x as f64, y as f64, 3.0);
                            for &idx in &nearby {
                                cell_rank += self.world.entities[idx].intel.rank;
                            }
                            let intensity = if !nearby.is_empty() {
                                ((cell_rank / nearby.len() as f32) / max_rank * 255.0) as u8
                            } else {
                                0
                            };
                            cell.set_bg(Color::Rgb(intensity / 2, 0, intensity));
                        }
                        4 => {
                            // VOCAL
                            let mut vocal_sum = 0.0;
                            let nearby = self.world.spatial_hash.query(x as f64, y as f64, 5.0);
                            for &idx in &nearby {
                                let dist = ((self.world.entities[idx].physics.x - x as f64)
                                    .powi(2)
                                    + (self.world.entities[idx].physics.y - y as f64).powi(2))
                                .sqrt();
                                vocal_sum += self.world.entities[idx].intel.last_vocalization
                                    * (1.0 - (dist / 5.0)).max(0.0) as f32;
                            }
                            let intensity = (vocal_sum.min(1.0) * 255.0) as u8;
                            cell.set_bg(Color::Rgb(intensity, intensity, 0));
                        }
                        _ => {
                            // NORMAL
                            let sm = self.world.social_grid[y as usize][x as usize];
                            if sm == 1 {
                                cell.set_bg(Color::Rgb(0, 0, 40));
                            } else if sm == 2 {
                                cell.set_bg(Color::Rgb(40, 0, 0));
                            }
                        }
                    }
                    if terrain.terrain_type != TerrainType::Plains {
                        cell.set_symbol(terrain.terrain_type.symbol().to_string().as_str());
                        cell.set_fg(terrain.terrain_type.color());
                    }
                }
            }
        }

        for food in &self.world.food {
            if let Some((x, y)) =
                Self::world_to_screen(f64::from(food.x), f64::from(food.y), area, self.screensaver)
            {
                let cell = buf.get_mut(x, y);
                cell.set_symbol(&food.symbol.to_string());
                cell.set_fg(Color::Rgb(
                    food.color_rgb.0,
                    food.color_rgb.1,
                    food.color_rgb.2,
                ));
            }
        }

        for entity in &self.world.entities {
            if let Some((x, y)) =
                Self::world_to_screen(entity.physics.x, entity.physics.y, area, self.screensaver)
            {
                let status = entity.status(
                    self.world.config.brain.activation_threshold,
                    self.world.tick,
                    self.world.config.metabolism.maturity_age,
                );
                let cell = buf.get_mut(x, y);
                cell.set_symbol(&entity.symbol_for_status(status).to_string());
                cell.set_fg(entity.color_for_status(status));

                if self.view_mode >= 2 {
                    if entity.intel.rank > 0.9 {
                        cell.set_bg(Color::Rgb(100, 100, 0));
                    } else if status == crate::model::state::entity::EntityStatus::Soldier {
                        cell.set_bg(Color::Rgb(80, 0, 0));
                    }
                }
                if entity.intel.bonded_to.is_some() {
                    cell.set_bg(Color::Rgb(80, 80, 0));
                }
            }
        }

        // Phase 51: Draw Symbiosis Bonds (Post-Process Line Rendering)
        for entity in &self.world.entities {
            if let Some(partner_id) = entity.intel.bonded_to {
                // Only draw if we have screen positions for both
                if let (Some(&(x1, y1)), Some(&(x2, y2))) = (
                    screen_positions.get(&entity.id),
                    screen_positions.get(&partner_id),
                ) {
                    // Bresenham's Line Algorithm
                    let mut x0 = x1 as i16;
                    let mut y0 = y1 as i16;
                    let x_end = x2 as i16;
                    let y_end = y2 as i16;

                    let dx = (x_end - x0).abs();
                    let dy = -(y_end - y0).abs();
                    let sx = if x0 < x_end { 1 } else { -1 };
                    let sy = if y0 < y_end { 1 } else { -1 };
                    let mut err = dx + dy;

                    // Draw max 10 steps
                    let mut steps = 0;
                    while steps < 10 {
                        if x0 == x_end && y0 == y_end {
                            break;
                        }
                        let e2 = 2 * err;
                        if e2 >= dy {
                            err += dy;
                            x0 += sx;
                        }
                        if e2 <= dx {
                            err += dx;
                            y0 += sy;
                        }

                        // Check bounds
                        if x0 >= inner.left() as i16
                            && x0 < inner.right() as i16
                            && y0 >= inner.top() as i16
                            && y0 < inner.bottom() as i16
                        {
                            let b_cell = buf.get_mut(x0 as u16, y0 as u16);
                            // Only draw over empty space or terrain, don't overwrite entities
                            if b_cell.symbol() == " " || b_cell.symbol() == "·" {
                                b_cell.set_symbol("·");
                                b_cell.set_fg(Color::Yellow);
                            }
                        }
                        steps += 1;
                    }
                }
            }
        }
    }
}
