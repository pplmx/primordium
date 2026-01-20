use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::widgets::{Block, Borders, Widget};

use crate::model::world::World;

pub struct WorldWidget<'a> {
    world: &'a World,
    screensaver: bool,
}

impl<'a> WorldWidget<'a> {
    pub fn new(world: &'a World, screensaver: bool) -> Self {
        Self { world, screensaver }
    }
}

impl<'a> Widget for WorldWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let inner_area = if self.screensaver {
            area
        } else {
            let block = Block::default()
                .title(format!("World (Tick: {})", self.world.tick))
                .borders(Borders::ALL);
            let inner = block.inner(area);
            block.render(area, buf);
            inner
        };

        // 1. Render Food (Green '*')
        for food in &self.world.food {
            let x = inner_area.x + food.x;
            let y = inner_area.y + food.y;

            if x >= inner_area.left()
                && x < inner_area.right()
                && y >= inner_area.top()
                && y < inner_area.bottom()
            {
                let cell = buf.get_mut(x, y);
                cell.set_symbol(&food.symbol.to_string());
                cell.set_fg(food.color);
            }
        }

        // 2. Optimized Entity Rendering
        for entity in &self.world.entities {
            let screen_x = inner_area.x as f64 + entity.x;
            let screen_y = inner_area.y as f64 + entity.y;

            let x = screen_x as u16;
            let y = screen_y as u16;

            if x >= inner_area.left()
                && x < inner_area.right()
                && y >= inner_area.top()
                && y < inner_area.bottom()
            {
                let cell = buf.get_mut(x, y);
                cell.set_symbol(&entity.symbol.to_string());
                cell.set_fg(entity.color());
            }
        }
    }
}
