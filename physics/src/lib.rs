mod utils;

use glam::DVec2;
use wasm_bindgen::prelude::*;
use web_sys::console;

const G: f64 = 6.67430e-11;
const STANDARD_GRAVITY: f64 = 9.80665;

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
    pub tick_time: f64,
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
            craft.speed += accel * self.cfg.tick_time;
            craft.position += craft.speed * self.cfg.tick_time;
            craft.consume_fuel(self.cfg.tick_time)
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
            y: self.position.y,
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
    pub y: f64,
}

#[wasm_bindgen]
pub struct Craft {
    dry_mass: f32,
    fuel_mass: f32,
    isp: f32,
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
            y: self.position.y,
        }
    }

    pub fn deltav(&self) -> f64 {
        let exhaust_vel = self.isp as f64 * STANDARD_GRAVITY;
        let mass_ratio = self.mass() / self.dry_mass as f64;
        exhaust_vel * mass_ratio.ln()
    }
}

impl Craft {
    fn mass(&self) -> f64 {
        (self.dry_mass + self.fuel_mass) as f64
    }

    fn thrust_accel(&self) -> DVec2 {
        if self.fuel_mass == 0.0 {
            return DVec2::new(0.0, 0.0);
        }

        let hdg: DVec2 = (self.heading as f64).sin_cos().into();
        let base_vec = DVec2::new((self.thrust * self.throttle) as f64, 0.0);
        let thrust = hdg.rotate(base_vec);
        thrust / self.mass()
    }

    /// Compute the consumed fuel from the expended delta-v in the given time
    fn consume_fuel(&mut self, time: f64) {
        // dv = isp * g * ln(m0/m1)
        // ln(m0/m1) = dv / (isp * g)
        // m0/m1 = e^(dv / (isp * g))
        // m1 = m0 / e^(dv / (isp * g))
        let dv = time * self.thrust as f64 * self.throttle as f64 / self.mass();
        let exhaust_velocity = self.isp as f64 * STANDARD_GRAVITY;
        let wet_final = self.mass() / std::f64::consts::E.powf(dv / exhaust_velocity);
        let fuel = wet_final as f32 - self.dry_mass;
        self.fuel_mass = if fuel > 0.0 { fuel } else { 0.0 };
    }
}

#[cfg(test)]
mod tests {
    use crate::Craft;

    #[test]
    fn fuel_comsumption() {
        let mut craft = Craft {
            dry_mass: 500.0,
            fuel_mass: 500.0,
            isp: 200.0,
            heading: 0.0,
            thrust: 2000.0,
            throttle: 1.0,
            position: (0.0, 0.0).into(),
            speed: (0.0, 0.0).into(),
        };
        craft.consume_fuel(0.5);
        assert!(craft.fuel_mass < 500.0)
    }

    #[test]
    fn deltav() {
        let mut craft = Craft {
            dry_mass: 500.0,
            fuel_mass: 500.0,
            isp: 200.0,
            heading: 0.0,
            thrust: 2000.0,
            throttle: 1.0,
            position: (0.0, 0.0).into(),
            speed: (0.0, 0.0).into(),
        };
        let dv_1 = craft.deltav();
        craft.consume_fuel(1.0);
        let dv_2 = craft.deltav();
        let dv_diff = dv_1 - dv_2;
        // i sure love testing for rounding errors
        assert!(dbg!((dv_diff - 2.0).abs()) < 1e-4);
    }
}
