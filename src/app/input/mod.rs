pub mod genetic_edit;
pub mod normal;
pub mod terrain_edit;

use crate::app::state::App;
use crossterm::event::KeyEvent;

impl App {
    pub fn handle_key(&mut self, key: KeyEvent) {
        self.handle_normal_key(key);
    }
}
