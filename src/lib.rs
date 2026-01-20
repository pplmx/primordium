#[cfg(target_arch = "wasm32")]
pub mod client;

// These must be available to server too!
// These must be available to server too!
pub mod app;
pub mod model;
pub mod ui;

#[cfg(target_arch = "wasm32")]
use crate::client::manager::NetworkManager;
use crate::model::config::AppConfig;
#[cfg(target_arch = "wasm32")]
use crate::model::environment::Environment;
use crate::model::environment::Environment;
use crate::model::world::World;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::model::network::NetMessage;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct Simulation {
    world: World,
    env: Environment,
    network: Option<NetworkManager>,
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
            network: None,
        })
    }

    pub fn connect(&mut self, url: &str) {
        self.network = Some(NetworkManager::new(url));
    }

    pub fn tick(&mut self) -> Result<(), JsValue> {
        self.world
            .update(&self.env)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Network Logic
        if let Some(net) = &self.network {
            // 1. Process incoming migrations
            for msg in net.pop_pending() {
                if let NetMessage::MigrateEntity {
                    dna,
                    energy,
                    generation,
                    ..
                } = msg
                {
                    self.world.import_migrant(dna, energy, generation);
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(&JsValue::from_str(
                        "Entity migrated into this universe!",
                    ));
                }
            }

            // 2. Check for outgoing migrations
            let mut migrants = Vec::new();
            self.world.entities.retain(|e| {
                let leaving = e.x < 1.0
                    || e.x > (self.world.width as f64 - 2.0)
                    || e.y < 1.0
                    || e.y > (self.world.height as f64 - 2.0);

                if leaving {
                    migrants.push(NetMessage::MigrateEntity {
                        dna: "DNA_PLACEHOLDER".to_string(), // In real impl, serialize brain
                        energy: e.energy,
                        generation: e.generation,
                        species_name: "Primordial".to_string(),
                    });
                }
                !leaving
            });

            for msg in migrants {
                net.send(&msg);
                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(&JsValue::from_str("Entity migrated to another universe!"));
            }
        }

        Ok(())
    }

    pub fn draw(&self, ctx: &web_sys::CanvasRenderingContext2d, width: f64, height: f64) {
        use crate::ui::web_renderer::WebRenderer;
        let renderer = WebRenderer::new(width, height, self.world.width, self.world.height);
        renderer.render(ctx, &self.world);
    }

    pub fn get_stats(&self) -> js_sys::Object {
        let obj = js_sys::Object::new();
        let _ = js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("tick"),
            &JsValue::from_f64(self.world.tick as f64),
        );
        let _ = js_sys::Reflect::set(
            &obj,
            &JsValue::from_str("entities"),
            &JsValue::from_f64(self.world.entities.len() as f64),
        );
        obj
    }
}
