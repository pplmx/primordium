pub mod app;
pub mod client;
pub mod model;
pub mod ui;

#[cfg(target_arch = "wasm32")]
use crate::client::manager::NetworkManager;
#[cfg(target_arch = "wasm32")]
use crate::model::state::environment::Environment;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::model::infra::network::NetMessage;

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct Simulation {
    world: model::world::World,
    env: model::state::environment::Environment,
    network: Option<crate::client::manager::NetworkManager>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl Simulation {
    pub fn new() -> Result<Simulation, JsValue> {
        console_error_panic_hook::set_once();

        let config = model::config::AppConfig::default();
        let world = model::world::World::new(config.world.initial_population, config.clone())
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        Ok(Simulation {
            world,
            env: model::state::environment::Environment::default(),
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
            // 1. Process incoming migrations (limited to 5 per tick for backpressure)
            for msg in net.pop_pending_limited(5) {
                if let NetMessage::MigrateEntity {
                    dna,
                    energy,
                    generation,
                    fingerprint,
                    checksum,
                    ..
                } = msg
                {
                    let _ =
                        self.world
                            .import_migrant(dna, energy, generation, &fingerprint, &checksum);
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(&JsValue::from_str(
                        "Entity migrated into this universe!",
                    ));
                }
            }

            // 2. Check for outgoing migrations
            let mut migrants = Vec::new();
            self.world.entities.retain(|e| {
                let leaving = e.physics.x < 1.0
                    || e.physics.x > (self.world.width as f64 - 2.0)
                    || e.physics.y < 1.0
                    || e.physics.y > (self.world.height as f64 - 2.0);

                if leaving {
                    let dna = e.intel.genotype.to_hex();
                    let energy = e.metabolism.energy as f32;
                    let generation = e.metabolism.generation;

                    use sha2::{Digest, Sha256};
                    let mut hasher = Sha256::new();
                    hasher.update(dna.as_bytes());
                    hasher.update(energy.to_be_bytes());
                    hasher.update(generation.to_be_bytes());
                    let checksum = hex::encode(hasher.finalize());

                    migrants.push(NetMessage::MigrateEntity {
                        dna,
                        energy,
                        generation,
                        species_name: e.name(),
                        fingerprint: self.world.config.fingerprint(),
                        checksum,
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
