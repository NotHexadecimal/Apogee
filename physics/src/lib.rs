mod utils;

use std::collections::VecDeque;

use nalgebra::{Rotation2, Vector2};
use wasm_bindgen::prelude::*;
use web_sys::console;

type DVec2 = Vector2<f64>;

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
                    .fold(craft.accel_vector(), |f1, f2| f1 + f2);
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

/// Exerts gravity on [Craft]s
#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct Planet {
    pub mass: f64,
    pub radius: f64,
    position: DVec2,
}

#[wasm_bindgen]
impl Planet {
    #[wasm_bindgen(constructor)]
    pub fn new(mass: f64, radius: f64, pos: AbiDVec2) -> Self {
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
    /// Computes the gravitational acceleration applied on an object of negligible mass
    fn gravity_accel_on(&self, pos: DVec2) -> DVec2 {
        let mut dist = self.position - pos;
        let accel_mod = self.mass * G / dist.magnitude().powi(2);
        dist.set_magnitude(accel_mod);
        dist
    }
}

// How do I pass this stuff by value to JS
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

impl From<AbiDVec2> for DVec2 {
    fn from(vec: AbiDVec2) -> Self {
        DVec2::new(vec.x, vec.y)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct VelPos {
    pub vel: DVec2,
    pub pos: DVec2,
}

impl From<VelPos> for (DVec2, DVec2) {
    fn from(value: VelPos) -> Self {
        (value.vel.into(), value.pos.into())
    }
}

/// Represents a spacecraft propelled by a reaction motor
#[wasm_bindgen]
#[derive(Debug, Default)]
pub struct Craft {
    pub dry_mass: f64,
    pub fuel_mass: f64,
    pub isp: f64,
    pub thrust: f64,
    position: DVec2,
    speed: DVec2,
    pub heading: f64,
    pub throttle: f64,
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

    /// Computes the craft's delta-v
    pub fn deltav(&self) -> f64 {
        let exhaust_vel = self.isp * STANDARD_GRAVITY;
        let mass_ratio = self.mass() / self.dry_mass;
        exhaust_vel * mass_ratio.ln()
    }

    // Not JS iterator compliant but should be good enough?
    pub fn trajectory_iter(&self) -> TrajectoryIter {
        TrajectoryIter {
            inner: &self.trajectory as *const _,
        }
    }
}

/// Can call a JS closure over items in the deque
#[wasm_bindgen]
pub struct TrajectoryIter {
    inner: *const VecDeque<VelPos>,
}

#[wasm_bindgen]
impl TrajectoryIter {
    /// Calls the provided JS closure for each element in the buffer
    ///
    /// The closure takes x and y coordinates, if any exception is caught the loop is stopped and
    /// the error is returned
    pub fn each_position(&self, f: &js_sys::Function) -> Result<(), JsValue> {
        let this = JsValue::null();
        for elem in unsafe { &*self.inner } {
            f.call2(
                &this,
                &JsValue::from(elem.pos.x),
                &JsValue::from(elem.pos.y),
            )?;
        }
        Ok(())
    }
}

impl Craft {
    /// Total craft mass
    fn mass(&self) -> f64 {
        self.dry_mass + self.fuel_mass
    }

    /// Returns the craft's acceleration vector
    fn accel_vector(&self) -> DVec2 {
        if self.fuel_mass == 0.0 {
            return DVec2::new(0.0, 0.0);
        }

        let thrust = self.thrust * self.throttle;
        Rotation2::new(self.heading) * Vector2::new(thrust / self.mass(), 0.0)
    }

    /// Compute the consumed fuel from the expended delta-v in the given time
    fn consume_fuel(&mut self, time: f64) {
        // flow_rate = F / (g_0 * Isp)

        let force = self.thrust * self.throttle;
        let exhaust_velocity = self.isp * STANDARD_GRAVITY;
        let flow_rate = dbg!(force) / dbg!(exhaust_velocity);

        self.fuel_mass = (self.fuel_mass - flow_rate * time).max(0.0)
    }

    /// Computes or extends the current trajectory
    fn populate_trajectory(&mut self, planets: &[Planet], timestep: f64, len: u64) {
        let start = if let Some(vp) = self.trajectory.back() {
            (*vp).into()
        } else {
            (self.speed, self.position)
        };
        let iter = std::iter::successors(Some(start), |(mut speed, position)| {
            let accel = planets
                .iter()
                .map(|p| p.gravity_accel_on(*position))
                .fold(Vector2::new(0.0, 0.0), |a, b| a + b);
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
