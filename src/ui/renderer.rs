use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Widget};

use crate::model::terrain::TerrainType;
use crate::model::world::World;

pub struct WorldWidget<'a> {
    world: &'a World,
    screensaver: bool,
}

impl<'a> WorldWidget<'a> {
    pub fn new(world: &'a World, screensaver: bool) -> Self {
        Self { world, screensaver }
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
                // Only render non-plains terrain to avoid clutter
                if terrain.terrain_type != TerrainType::Plains {
                    let screen_x = inner.x + x;
                    let screen_y = inner.y + y;
                    if screen_x < inner.right() && screen_y < inner.bottom() {
                        let cell = buf.get_mut(screen_x, screen_y);
                        cell.set_symbol(&terrain.terrain_type.symbol().to_string());
                        cell.set_fg(terrain.terrain_type.color());
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
                cell.set_fg(food.color);
            }
        }

        // 2. Optimized Entity Rendering
        for entity in &self.world.entities {
            if let Some((x, y)) = Self::world_to_screen(entity.x, entity.y, area, self.screensaver)
            {
                let cell = buf.get_mut(x, y);
                cell.set_symbol(&entity.symbol.to_string());
                cell.set_fg(entity.color());
            }
        }
    }
}
