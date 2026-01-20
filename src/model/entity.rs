use rand::Rng;
use ratatui::style::Color;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Entity {
    pub id: Uuid,
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub color: Color,
    pub symbol: char,
    pub energy: f64,
    pub max_energy: f64,
    pub generation: u32,
}

impl Entity {
    pub fn new(x: f64, y: f64) -> Self {
        let mut rng = rand::thread_rng();

        // Random velocity between -0.5 and 0.5
        let vx = rng.gen_range(-0.5..0.5);
        let vy = rng.gen_range(-0.5..0.5);

        // Random bright RGB color
        let r = rng.gen_range(100..255);
        let g = rng.gen_range(100..255);
        let b = rng.gen_range(100..255);
        let color = Color::Rgb(r, g, b);

        Self {
            id: Uuid::new_v4(),
            x,
            y,
            vx,
            vy,
            color,
            symbol: '●',
            energy: 100.0,
            max_energy: 200.0,
            generation: 1,
        }
    }

    pub fn reproduce(&mut self) -> Self {
        let mut rng = rand::thread_rng();

        // Split energy
        let child_energy = self.energy / 2.0;
        self.energy = child_energy;

        // Mutate velocity (±10%)
        let mut child_vx = self.vx + rng.gen_range(-0.05..0.05);
        let mut child_vy = self.vy + rng.gen_range(-0.05..0.05);

        // Clamp velocity to reasonable limits (-1.0 to 1.0) to prevent explosion
        child_vx = child_vx.max(-1.0).min(1.0);
        child_vy = child_vy.max(-1.0).min(1.0);

        // Mutate Color (±15)
        let (r, g, b) = match self.color {
            Color::Rgb(r, g, b) => (r, g, b),
            _ => (255, 255, 255), // Fallback if somehow not RGB
        };

        let mut mutate_color = |c: u8| -> u8 {
            let change = rng.gen_range(-15..=15);
            (c as i16 + change).max(0).min(255) as u8
        };

        let child_color = Color::Rgb(mutate_color(r), mutate_color(g), mutate_color(b));

        Self {
            id: Uuid::new_v4(),
            x: self.x, // Spawn at parent location (will move next tick)
            y: self.y,
            vx: child_vx,
            vy: child_vy,
            color: child_color,
            symbol: '●',
            energy: child_energy,
            max_energy: self.max_energy,
            generation: self.generation + 1,
        }
    }
}
