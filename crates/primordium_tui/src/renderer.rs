use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Color;
use ratatui::widgets::{Block, Borders, Widget};
use std::collections::HashMap;

use primordium_core::snapshot::{EntitySnapshot, WorldSnapshot};
use primordium_core::terrain::{TerrainLogic, TerrainType};
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
        if status == EntityStatus::Starving {
            return Color::Rgb(255, 0, 0);
        }
        if status == EntityStatus::Infected {
            return Color::Rgb(154, 205, 50);
        }

        let base_color = match entity.specialization {
            Some(primordium_data::Specialization::Soldier) => Color::Rgb(255, 50, 50),
            Some(primordium_data::Specialization::Engineer) => Color::Cyan,
            Some(primordium_data::Specialization::Provider) => Color::Yellow,
            None => Color::Rgb(100, 255, 100),
        };

        if entity.is_larva {
            match base_color {
                Color::Rgb(r, g, b) => Color::Rgb(r / 2, g / 2, b / 2),
                _ => base_color,
            }
        } else {
            base_color
        }
    }

    pub fn symbol_for_status(entity: &EntitySnapshot) -> char {
        if entity.status == EntityStatus::InTransit {
            return '✈';
        }
        if entity.status == EntityStatus::Starving {
            return '†';
        }
        if entity.status == EntityStatus::Infected {
            return '☣';
        }
        if entity.status == EntityStatus::Hunting {
            return '♦';
        }
        if entity.status == EntityStatus::Mating {
            return '♥';
        }
        if entity.status == EntityStatus::Sharing {
            return '♣';
        }
        if entity.status == EntityStatus::Bonded {
            return '⚭';
        }

        match (entity.specialization, entity.is_larva) {
            (Some(primordium_data::Specialization::Soldier), true) => '△',
            (Some(primordium_data::Specialization::Soldier), false) => '▲',
            (Some(primordium_data::Specialization::Engineer), true) => '◇',
            (Some(primordium_data::Specialization::Engineer), false) => '◈',
            (Some(primordium_data::Specialization::Provider), true) => '○',
            (Some(primordium_data::Specialization::Provider), false) => '◎',
            (None, true) => '·',
            (None, false) => '●',
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

        let mut screen_positions = HashMap::with_capacity(self.snapshot.entities.len());

        let left_f = inner.left() as f64 - inner.x as f64;
        let top_f = inner.top() as f64 - inner.y as f64;
        let right_f = inner.right() as f64 - inner.x as f64;
        let bottom_f = inner.bottom() as f64 - inner.y as f64;

        for entity in &self.snapshot.entities {
            if entity.x >= left_f && entity.x < right_f && entity.y >= top_f && entity.y < bottom_f
            {
                if let Some((x, y)) =
                    Self::world_to_screen(entity.x, entity.y, area, self.screensaver)
                {
                    screen_positions.insert(entity.id, (x, y));

                    let status = entity.status;
                    let cell = &mut buf[(x, y)];
                    cell.set_symbol(&Self::symbol_for_status(entity).to_string());
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
        }

        let map_w = self.snapshot.terrain.width;
        let map_h = self.snapshot.terrain.height;

        let start_x = 0;
        let end_x = inner.width.min(map_w);
        let start_y = 0;
        let end_y = inner.height.min(map_h);

        for y in start_y..end_y {
            for x in start_x..end_x {
                let terrain = self.snapshot.terrain.get_cell(x, y);
                let screen_x = inner.x + x;
                let screen_y = inner.y + y;

                if screen_x < inner.right() && screen_y < inner.bottom() {
                    let cell = &mut buf[(screen_x, screen_y)];

                    if cell.symbol() != " " {
                        continue;
                    }

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
                cell.set_symbol(&Self::symbol_for_status(entity).to_string());
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

#[cfg(test)]
mod tests {
    use super::*;
    use primordium_core::snapshot::EntitySnapshot;
    use primordium_data::Specialization;

    fn create_dummy_entity() -> EntitySnapshot {
        EntitySnapshot {
            id: uuid::Uuid::new_v4(),
            name: "Test".to_string(),
            x: 0.0,
            y: 0.0,
            r: 0,
            g: 0,
            b: 0,
            energy: 100.0,
            max_energy: 100.0,
            generation: 1,
            age: 0,
            offspring: 0,
            lineage_id: uuid::Uuid::new_v4(),
            rank: 0.0,
            status: EntityStatus::Foraging,
            trophic_potential: 0.5,
            bonded_to: None,
            last_vocalization: 0.0,
            last_activations: std::collections::HashMap::new(),
            weight_deltas: std::collections::HashMap::new(),
            genotype_hex: None,
            specialization: None,
            is_larva: false,
        }
    }

    #[test]
    fn test_color_for_status() {
        let mut entity = create_dummy_entity();

        // Starving
        assert_eq!(
            WorldWidget::color_for_status(&entity, EntityStatus::Starving),
            Color::Rgb(255, 0, 0)
        );

        // Normal
        assert_eq!(
            WorldWidget::color_for_status(&entity, EntityStatus::Foraging),
            Color::Rgb(100, 255, 100)
        );

        // Specialization
        entity.specialization = Some(Specialization::Soldier);
        assert_eq!(
            WorldWidget::color_for_status(&entity, EntityStatus::Foraging),
            Color::Rgb(255, 50, 50)
        );
    }

    #[test]
    fn test_symbol_for_status() {
        let mut entity = create_dummy_entity();

        // Adult Foraging
        assert_eq!(WorldWidget::symbol_for_status(&entity), '●');

        // Larva Foraging
        entity.is_larva = true;
        assert_eq!(WorldWidget::symbol_for_status(&entity), '·');

        // Starving
        entity.status = EntityStatus::Starving;
        assert_eq!(WorldWidget::symbol_for_status(&entity), '†');

        // Hunting
        entity.status = EntityStatus::Hunting;
        assert_eq!(WorldWidget::symbol_for_status(&entity), '♦');

        // Soldier (Normal state overrides status if it's just Foraging, but special statuses like Hunting override Soldier)
        entity.status = EntityStatus::Hunting;
        entity.is_larva = false;
        entity.specialization = Some(Specialization::Soldier);
        assert_eq!(WorldWidget::symbol_for_status(&entity), '♦');

        // Soldier (Foraging)
        entity.status = EntityStatus::Foraging;
        assert_eq!(WorldWidget::symbol_for_status(&entity), '▲');
    }

    #[test]
    fn test_terrain_mappings() {
        assert_eq!(WorldWidget::symbol_for_terrain(TerrainType::Mountain), '▲');
        assert_eq!(
            WorldWidget::color_for_terrain(TerrainType::River),
            Color::Rgb(70, 130, 180)
        );
    }
}
