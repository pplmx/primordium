use crate::app::state::App;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use primordium_tui::renderer::WorldWidget;
use rand::Rng;
use ratatui::style::Color;
use std::sync::Arc;

impl App {
    pub fn handle_mouse(&mut self, mouse: MouseEvent) {
        if self.show_brain && mouse.column >= self.last_sidebar_rect.x {
            if matches!(mouse.kind, MouseEventKind::Down(MouseButton::Left)) {
                let relative_y = mouse.row.saturating_sub(self.last_sidebar_rect.y + 1);
                let offset = self.gene_editor_offset;
                use primordium_data::GeneType;
                self.focused_gene = if relative_y == offset {
                    Some(GeneType::Trophic)
                } else if relative_y == offset + 1 {
                    Some(GeneType::Sensing)
                } else if relative_y == offset + 2 {
                    Some(GeneType::Speed)
                } else if relative_y == offset + 3 {
                    Some(GeneType::MaxEnergy)
                } else if relative_y == offset + 4 {
                    Some(GeneType::ReproInvest)
                } else if relative_y == offset + 5 {
                    Some(GeneType::Maturity)
                } else {
                    None
                };
            }
            return;
        }

        match mouse.kind {
            MouseEventKind::Down(MouseButton::Left) | MouseEventKind::Drag(MouseButton::Left) => {
                self.handle_left_click_or_drag(mouse);
            }
            MouseEventKind::Down(MouseButton::Right) => {
                self.handle_right_click(mouse);
            }
            _ => {}
        }
    }

    fn handle_left_click_or_drag(&mut self, mouse: MouseEvent) {
        if let Some((wx, wy)) = WorldWidget::screen_to_world(
            mouse.column,
            mouse.row,
            self.last_world_rect,
            self.screensaver,
        ) {
            let painted = if matches!(mouse.kind, MouseEventKind::Drag(MouseButton::Left)) {
                true
            } else {
                let mut closest_id = None;
                for (_handle, (identity, pos)) in self
                    .world
                    .ecs
                    .query::<(&primordium_data::Identity, &crate::model::state::Position)>()
                    .iter()
                {
                    let dx = pos.x - wx;
                    let dy = pos.y - wy;
                    if dx * dx + dy * dy < 4.0 {
                        closest_id = Some(identity.id);
                        break;
                    }
                }

                if let Some(id) = closest_id {
                    self.selected_entity = Some(id);
                    self.show_brain = true;
                    false
                } else {
                    true
                }
            };

            if painted {
                if self.is_social_brush {
                    let ix = (wx as usize).min(self.world.width as usize - 1);
                    let iy = (wy as usize).min(self.world.height as usize - 1);
                    let width = self.world.width as usize;
                    Arc::make_mut(&mut self.world.social_grid)[iy * width + ix] = self.social_brush;
                } else {
                    Arc::make_mut(&mut self.world.terrain).set_cell_type(
                        wx as u16,
                        wy as u16,
                        self.brush_type,
                    );
                }
            }
        }
    }

    fn handle_right_click(&mut self, mouse: MouseEvent) {
        if let Some((wx, wy)) = WorldWidget::screen_to_world(
            mouse.column,
            mouse.row,
            self.last_world_rect,
            self.screensaver,
        ) {
            use crate::model::state::{MetabolicNiche, Position};
            use primordium_data::Food;
            let n_type = rand::thread_rng().gen_range(0.0..1.0);
            self.world.ecs.spawn((
                Food::new(wx as u16, wy as u16, n_type),
                Position { x: wx, y: wy },
                MetabolicNiche(n_type),
            ));
            self.world.food_dirty = true;
            self.event_log
                .push_back(("Divine Food Injected".to_string(), Color::Green));
        }
    }
}
