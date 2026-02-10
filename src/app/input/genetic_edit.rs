use crate::app::state::App;

impl App {
    pub fn handle_increment_key(&mut self) {
        if self.show_brain && self.focused_gene.is_some() {
            if let (Some(id), Some(gene)) = (self.selected_entity, self.focused_gene) {
                let delta = match gene {
                    primordium_data::GeneType::Trophic => 0.05,
                    primordium_data::GeneType::Sensing => 0.5,
                    primordium_data::GeneType::Speed => 0.1,
                    primordium_data::GeneType::MaxEnergy => 10.0,
                    primordium_data::GeneType::ReproInvest => 0.05,
                    primordium_data::GeneType::Maturity => 0.1,
                };
                self.world.apply_genetic_edit(id, gene, delta);
            }
        } else {
            self.time_scale = (self.time_scale + 0.5).min(4.0);
        }
    }

    pub fn handle_decrement_key(&mut self) {
        if self.show_brain && self.focused_gene.is_some() {
            if let (Some(id), Some(gene)) = (self.selected_entity, self.focused_gene) {
                let delta = match gene {
                    primordium_data::GeneType::Trophic => -0.05,
                    primordium_data::GeneType::Sensing => -0.5,
                    primordium_data::GeneType::Speed => -0.1,
                    primordium_data::GeneType::MaxEnergy => -10.0,
                    primordium_data::GeneType::ReproInvest => -0.05,
                    primordium_data::GeneType::Maturity => -0.1,
                };
                self.world.apply_genetic_edit(id, gene, delta);
            }
        } else {
            self.time_scale = (self.time_scale - 0.5).max(0.5);
        }
    }
}
