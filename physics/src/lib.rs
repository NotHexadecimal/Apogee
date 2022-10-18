mod utils;

use glam::DVec2;
use wasm_bindgen::prelude::*;
use web_sys::console;

const G: f64 = 6.67430e-11;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen(start)]
pub fn main() {
    utils::set_panic_hook();
    console::log_1(&"Done loading WASM blob".into());
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Config {
    pub tick_time: f32,
    // TODO
    // pub precompute_ticks: u64,
}

#[wasm_bindgen]
pub struct Simulation {
    pub cfg: Config,
    planets: Vec<Planet>,
    crafts: Vec<Craft>,
}

#[wasm_bindgen]
impl Simulation {
    pub fn tick(&mut self) {
        for craft in self.crafts.iter_mut() {
            let accel: DVec2 = self
                .planets
                .iter()
                .map(|p| p.gravity_accel_on(craft.position))
                .fold(craft.thrust_accel(), |f1, f2| f1 + f2);
            craft.speed += accel * self.cfg.tick_time as f64;
            craft.position += craft.speed * self.cfg.tick_time as f64;
        }
    }
}

#[wasm_bindgen]
pub struct Planet {
    mass: f32,
    pub radius: f32,
    position: DVec2,
}

#[wasm_bindgen]
impl Planet {
    pub fn position(&self) -> AbiPosition {
        AbiPosition {
            x: self.position.x,
            y: self.position.y
        }
    }
}

impl Planet {
    fn gravity_accel_on(&self, pos: DVec2) -> DVec2 {
        let dist = self.position - pos;
        let accel_mod = self.mass as f64 * G / dist.length().powi(2);
        dist.normalize().rotate(DVec2::new(accel_mod, 0.0))
    }
}

#[wasm_bindgen]
pub struct AbiPosition {
    pub x: f64,
    pub y: f64
}

#[wasm_bindgen]
pub struct Craft {
    dry_mass: f32,
    fuel_mass: f32,
    // TODO: deltav and fuel comsumption simulation
    // isp: f32,
    thrust: f32,
    position: DVec2,
    speed: DVec2,
    pub heading: f32,
    pub throttle: f32,
    // TODO
    // trajectory: VecDeque<DVec2>
}

#[wasm_bindgen]
impl Craft {
    pub fn position(&self) -> AbiPosition {
        AbiPosition {
            x: self.position.x,
            y: self.position.y
        }
    }
}

impl Craft {
    fn mass(&self) -> f64 {
        (self.dry_mass + self.fuel_mass) as f64
    }

    fn thrust_accel(&self) -> DVec2 {
        let hdg: DVec2 = (self.heading as f64).sin_cos().into();
        let base_vec = DVec2::new((self.thrust * self.throttle) as f64, 0.0);
        let thrust = hdg.rotate(base_vec);
        thrust / self.mass()
    }
}
