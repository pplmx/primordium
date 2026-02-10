use super::TerrainGrid;

impl TerrainGrid {
    pub fn trigger_dust_bowl(&mut self, duration: u32) {
        self.dust_bowl_timer = duration;
    }
}
