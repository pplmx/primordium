pub mod app;
pub mod model;
pub mod ui;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use crate::model::world::World;
#[cfg(target_arch = "wasm32")]
use crate::model::config::AppConfig;
#[cfg(target_arch = "wasm32")]
use crate::model::environment::Environment;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct Simulation {
    world: World,
    env: Environment,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl Simulation {
    pub fn new() -> Result<Simulation, JsValue> {
        console_error_panic_hook::set_once();

        let config = AppConfig::default();
        let world = World::new(config.world.initial_population, config.clone())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(Simulation {
            world,
            env: Environment::default(),
        })
    }

    pub fn tick(&mut self) -> Result<(), JsValue> {
        self.world.update(&self.env)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    pub fn draw(&self, ctx: &web_sys::CanvasRenderingContext2d, width: f64, height: f64) {
        use crate::ui::web_renderer::WebRenderer;
        let renderer = WebRenderer::new(width, height, self.world.width, self.world.height);
        renderer.render(ctx, &self.world);
    }

    pub fn get_stats(&self) -> js_sys::Object {
        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("tick"), &JsValue::from_f64(self.world.tick as f64));
        let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("entities"), &JsValue::from_f64(self.world.entities.len() as f64));
        obj
    }
}
