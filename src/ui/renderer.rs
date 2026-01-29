use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::{Block, Borders, Widget};
use std::collections::HashMap;

use crate::model::snapshot::{EntitySnapshot, WorldSnapshot};
use crate::model::terrain::{TerrainLogic, TerrainType};
use primordium_data::EntityStatus;

pub struct WorldWidget<'a> {
    snapshot: &'a WorldSnapshot,
    screensaver: bool,
    view_mode: u8,
}

impl<'a> WorldWidget<'a> {
    pub fn new(snapshot: &'a WorldSnapshot, screensaver: bool, view_mode: u8) -> Self {
        Self {
            snapshot,
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

    pub fn color_for_status(entity: &EntitySnapshot, status: EntityStatus) -> Color {
        match status {
            EntityStatus::Starving => Color::Rgb(150, 50, 50),
            EntityStatus::Infected => Color::Rgb(154, 205, 50),
            EntityStatus::Larva => Color::Rgb(180, 180, 180),
            EntityStatus::Juvenile => Color::Rgb(200, 200, 255),
            EntityStatus::Sharing => Color::Rgb(100, 200, 100),
            EntityStatus::Mating => Color::Rgb(255, 105, 180),
            EntityStatus::Hunting => Color::Rgb(255, 69, 0),
            EntityStatus::Foraging => Color::Rgb(entity.r, entity.g, entity.b),
            EntityStatus::Soldier => Color::Red,
            EntityStatus::Bonded => Color::Rgb(255, 215, 0),
            EntityStatus::InTransit => Color::Rgb(150, 150, 150),
        }
    }

    pub fn symbol_for_status(status: EntityStatus) -> char {
        match status {
            EntityStatus::Starving => '†',
            EntityStatus::Infected => '☣',
            EntityStatus::Larva => '⋯',
            EntityStatus::Juvenile => '◦',
            EntityStatus::Sharing => '♣',
            EntityStatus::Mating => '♥',
            EntityStatus::Hunting => '♦',
            EntityStatus::Foraging => '●',
            EntityStatus::Soldier => '⚔',
            EntityStatus::Bonded => '⚭',
            EntityStatus::InTransit => '✈',
        }
    }

    pub fn symbol_for_terrain(t: TerrainType) -> char {
        t.symbol()
    }

    pub fn color_for_terrain(t: TerrainType) -> Color {
        match t {
            TerrainType::Plains => Color::Reset,
            TerrainType::Mountain => Color::Rgb(100, 100, 100),
            TerrainType::River => Color::Rgb(70, 130, 180),
            TerrainType::Oasis => Color::Rgb(50, 205, 50),
            TerrainType::Barren => Color::Rgb(139, 69, 19),
            TerrainType::Wall => Color::Rgb(60, 60, 60),
            TerrainType::Forest => Color::Rgb(34, 139, 34),
            TerrainType::Desert => Color::Rgb(210, 180, 140),
            TerrainType::Nest => Color::Rgb(255, 215, 0),
            TerrainType::Outpost => Color::Rgb(255, 69, 0),
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
                .title(format!("World (Tick: {})", self.snapshot.tick))
                .borders(Borders::ALL);
            block.render(area, buf);
        }

        let inner = Self::get_inner_area(area, self.screensaver);

        let mut screen_positions = HashMap::new();
        for entity in &self.snapshot.entities {
            if let Some((x, y)) = Self::world_to_screen(entity.x, entity.y, area, self.screensaver)
            {
                screen_positions.insert(entity.id, (x, y));
            }
        }

        for y in 0..inner.height.min(self.snapshot.terrain.height) {
            for x in 0..inner.width.min(self.snapshot.terrain.width) {
                let terrain = self.snapshot.terrain.get_cell(x, y);
                let screen_x = inner.x + x;
                let screen_y = inner.y + y;
                if screen_x < inner.right() && screen_y < inner.bottom() {
                    let cell = &mut buf[(screen_x, screen_y)];

                    match self.view_mode {
                        1 => {
                            let f = terrain.fertility;
                            cell.set_bg(Color::Rgb(
                                (255.0 * (1.0 - f)) as u8 / 4,
                                (255.0 * f) as u8 / 2,
                                0,
                            ));
                        }
                        2 => {
                            let sm = self.snapshot.social_grid
                                [(y as usize * self.snapshot.width as usize) + x as usize];
                            if sm == 1 {
                                cell.set_bg(Color::Rgb(0, 0, 100));
                            } else if sm == 2 {
                                cell.set_bg(Color::Rgb(100, 0, 0));
                            } else {
                                cell.set_bg(Color::Rgb(20, 20, 20));
                            }
                        }
                        3 => {
                            let val = self.snapshot.rank_grid
                                [(y as usize * self.snapshot.width as usize) + x as usize];
                            let intensity = (val.min(1.0) * 255.0) as u8;
                            cell.set_bg(Color::Rgb(intensity / 2, 0, intensity));
                        }
                        4 => {
                            let sound_val = self.snapshot.sound.get_cell(x, y);
                            let intensity = (sound_val.min(1.0) * 255.0) as u8;
                            cell.set_bg(Color::Rgb(intensity, intensity, 0));
                        }
                        _ => {
                            let sm = self.snapshot.social_grid
                                [(y as usize * self.snapshot.width as usize) + x as usize];
                            if sm == 1 {
                                cell.set_bg(Color::Rgb(0, 0, 40));
                            } else if sm == 2 {
                                cell.set_bg(Color::Rgb(40, 0, 0));
                            }
                        }
                    }
                    if terrain.terrain_type != TerrainType::Plains {
                        cell.set_symbol(
                            Self::symbol_for_terrain(terrain.terrain_type)
                                .to_string()
                                .as_str(),
                        );
                        cell.set_fg(Self::color_for_terrain(terrain.terrain_type));
                    }
                }
            }
        }

        for food in &self.snapshot.food {
            if let Some((x, y)) =
                Self::world_to_screen(f64::from(food.x), f64::from(food.y), area, self.screensaver)
            {
                let cell = &mut buf[(x, y)];
                cell.set_symbol(&food.symbol.to_string());
                cell.set_fg(Color::Rgb(
                    food.color_rgb.0,
                    food.color_rgb.1,
                    food.color_rgb.2,
                ));
            }
        }

        for entity in &self.snapshot.entities {
            if let Some((x, y)) = Self::world_to_screen(entity.x, entity.y, area, self.screensaver)
            {
                let status = entity.status;
                let cell = &mut buf[(x, y)];
                cell.set_symbol(&Self::symbol_for_status(status).to_string());
                cell.set_fg(Self::color_for_status(entity, status));
                if self.view_mode >= 2 {
                    if entity.rank > 0.9 {
                        cell.set_bg(Color::Rgb(100, 100, 0));
                    } else if status == EntityStatus::Soldier {
                        cell.set_bg(Color::Rgb(80, 0, 0));
                    }
                }
                if entity.bonded_to.is_some() {
                    cell.set_bg(Color::Rgb(80, 80, 0));
                }
            }
        }

        for entity in &self.snapshot.entities {
            if let Some(partner_id) = entity.bonded_to {
                if let (Some(&(x1, y1)), Some(&(x2, y2))) = (
                    screen_positions.get(&entity.id),
                    screen_positions.get(&partner_id),
                ) {
                    let mut x0 = x1 as i16;
                    let mut y0 = y1 as i16;
                    let x_end = x2 as i16;
                    let y_end = y2 as i16;
                    let dx = (x_end - x0).abs();
                    let dy = -(y_end - y0).abs();
                    let sx = if x0 < x_end { 1 } else { -1 };
                    let sy = if y0 < y_end { 1 } else { -1 };
                    let mut err = dx + dy;
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
                        if x0 >= inner.left() as i16
                            && x0 < inner.right() as i16
                            && y0 >= inner.top() as i16
                            && y0 < inner.bottom() as i16
                        {
                            let b_cell = &mut buf[(x0 as u16, y0 as u16)];
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
