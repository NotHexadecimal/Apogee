mod utils;

use std::collections::VecDeque;

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
#[derive(Debug, Default, Clone, Copy)]
pub struct Config {
    tick_time: f64,
    prediction_steps: u64,
}

#[wasm_bindgen]
impl Config {
    #[wasm_bindgen(constructor)]
    pub fn new(tick_time: f64, prediction_steps: u64) -> Self {
        Self {
            tick_time,
            prediction_steps,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct Simulation {
    pub cfg: Config,
    planets: Vec<Planet>,
    crafts: Vec<Craft>,
}

#[wasm_bindgen]
impl Simulation {
    #[wasm_bindgen(constructor)]
    pub fn new(cfg: Config) -> Self {
        Self {
            cfg,
            ..Default::default()
        }
    }

    /// Adds a planet to the simulation and recomputes the ships' trajectory
    pub fn add_planet(&mut self, planet: Planet) {
        self.planets.push(planet);
        self.recompute_craft_trajectories()
    }

    /// Adds a spacecraft to the simulation
    pub fn add_craft(&mut self, craft: Craft) {
        self.crafts.push(craft);
    }

    /// Advances the simulation by the configured delta-time
    pub fn tick(&mut self) {
        for craft in self.crafts.iter_mut() {
            if craft.throttle == 0.0 {
                craft.populate_trajectory(
                    &self.planets,
                    self.cfg.tick_time,
                    self.cfg.prediction_steps + 1,
                );
                (craft.speed, craft.position) = craft.trajectory.pop_front().unwrap().into();
            } else {
                let accel: DVec2 = self
                    .planets
                    .iter()
                    .map(|p| p.gravity_accel_on(craft.position))
                    .fold(craft.thrust_accel(), |f1, f2| f1 + f2);
                craft.speed += accel * self.cfg.tick_time;
                craft.position += craft.speed * self.cfg.tick_time;
                craft.consume_fuel(self.cfg.tick_time);

                craft.trajectory.clear();
                craft.populate_trajectory(
                    &self.planets,
                    self.cfg.tick_time,
                    self.cfg.prediction_steps,
                );
            }
        }
    }

    pub fn set_tick_time(&mut self, tick_time: f64) {
        self.cfg.tick_time = tick_time;
        self.recompute_craft_trajectories()
    }

    fn recompute_craft_trajectories(&mut self) {
        for craft in &mut self.crafts {
            craft.trajectory.clear();
            craft.populate_trajectory(&self.planets, self.cfg.tick_time, self.cfg.prediction_steps)
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct Planet {
    pub mass: f32,
    pub radius: f32,
    position: DVec2,
}

#[wasm_bindgen]
impl Planet {
    #[wasm_bindgen(constructor)]
    pub fn new(mass: f32, radius: f32, pos: AbiDVec2) -> Self {
        Self {
            mass,
            radius,
            position: pos.into(),
        }
    }

    #[wasm_bindgen(getter)]
    pub fn position(&self) -> AbiDVec2 {
        self.position.into()
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
#[derive(Debug, Default, Clone, Copy)]
pub struct AbiDVec2 {
    pub x: f64,
    pub y: f64,
}

impl From<DVec2> for AbiDVec2 {
    fn from(vec: DVec2) -> Self {
        Self { x: vec.x, y: vec.y }
    }
}

impl Into<DVec2> for AbiDVec2 {
    fn into(self) -> DVec2 {
        DVec2 {
            x: self.x,
            y: self.y,
        }
    }
}

#[wasm_bindgen]
#[derive(Debug, Default, Clone, Copy)]
pub struct VelPos {
    pub vel: AbiDVec2,
    pub pos: AbiDVec2,
}

impl Into<(DVec2, DVec2)> for VelPos {
    fn into(self) -> (DVec2, DVec2) {
        (self.vel.into(), self.pos.into())
    }
}

#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct Craft {
    pub dry_mass: f32,
    pub fuel_mass: f32,
    pub isp: f32,
    pub thrust: f32,
    position: DVec2,
    speed: DVec2,
    pub heading: f32,
    pub throttle: f32,
    // (speed, position)
    // I'd rather do without the AbiDVec2 conversions but glam doesn't do wasm_bindgen on its types
    trajectory: VecDeque<VelPos>,
}

#[wasm_bindgen]
impl Craft {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Default::default()
    }

    #[wasm_bindgen(getter)]
    pub fn position(&self) -> AbiDVec2 {
        self.position.into()
    }

    #[wasm_bindgen(setter)]
    pub fn set_position(&mut self, pos: AbiDVec2) {
        self.position = pos.into()
    }

    #[wasm_bindgen(getter)]
    pub fn speed(&self) -> AbiDVec2 {
        self.speed.into()
    }

    #[wasm_bindgen(setter)]
    pub fn set_speed(&mut self, vel: AbiDVec2) {
        self.speed = vel.into()
    }

    pub fn deltav(&self) -> f64 {
        let exhaust_vel = self.isp as f64 * STANDARD_GRAVITY;
        let mass_ratio = self.mass() / self.dry_mass as f64;
        exhaust_vel * mass_ratio.ln()
    }

    pub fn trajectory_ptr(&mut self) -> *const VelPos {
        self.trajectory.make_contiguous().as_ptr()
    }

    pub fn trajectory_len(&self) -> usize {
        self.trajectory.len()
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

    fn populate_trajectory(&mut self, planets: &[Planet], timestep: f64, len: u64) {
        let start = if let Some(vp) = self.trajectory.back() {
            (*vp).into()
        } else {
            (self.speed, self.position)
        };
        let iter = std::iter::successors(Some(start), |(mut speed, position)| {
            let accel: DVec2 = planets
                .iter()
                .map(|p| p.gravity_accel_on(*position))
                .fold((0.0, 0.0).into(), |a, b| a + b);
            speed += accel * timestep;
            Some((speed, *position + speed * timestep))
        })
        .map(|(vel, pos)| VelPos {
            vel: vel.into(),
            pos: pos.into(),
        })
        .take(len as usize - self.trajectory.len());

        self.trajectory.extend(iter);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

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
            trajectory: VecDeque::new(),
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
            trajectory: VecDeque::new(),
        };
        let dv_1 = craft.deltav();
        craft.consume_fuel(1.0);
        let dv_2 = craft.deltav();
        let dv_diff = dv_1 - dv_2;
        // i sure love testing for rounding errors
        assert!(dbg!((dv_diff - 2.0).abs()) < 1e-4);
    }
}
