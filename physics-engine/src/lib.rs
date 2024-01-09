use geometry::Point;
use levels::Level;
use physics::{compute, shape, DisplayMessage, WithColor};
use serde::{Deserialize, Serialize};
use tsify::Tsify;
use wasm_bindgen::prelude::*;

mod geometry;
mod levels;
mod physics;

#[wasm_bindgen]
pub struct Engine(Box<physics::Engine>);

#[derive(Serialize, Deserialize, Tsify, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Polygon {
    vertices: Vec<Point>,
}

#[wasm_bindgen]
extern "C" {
    pub fn alert(message: &str);
}

#[wasm_bindgen]
impl Engine {
    pub fn create(init: levels::Level) -> Self {
        console_error_panic_hook::set_once();
        Engine(Box::new(physics::Engine::new(init)))
    }

    pub fn run_iteration(&mut self, time_step_microseconds: f64) -> DisplayMessage {
        self.0.run_iteration(time_step_microseconds)
    }

    pub fn add_circle(&mut self, x: f64, y: f64, radius: f64) {
        self.0
            .add_circle(shape::Circle::new(geometry::Point(x, y), radius))
    }

    pub fn add_polygon(&mut self, polygon: Polygon) {
        self.0.add_polygon(compute::hull::<24>(
            polygon
                .vertices
                .into_iter()
                .map(|Point(x, y)| geometry::Point(x as f64, y as f64)),
        ))
    }

    pub fn erase_at(&mut self, x: f64, y: f64) {
        self.0.erase_at(geometry::Point(x, y));
    }

    pub fn add_hinge(&mut self, x: f64, y: f64) {
        self.0.add_hinge(geometry::Point(x, y));
    }

    pub fn add_rigid(&mut self, x: f64, y: f64) {
        self.0.add_rigid(geometry::Point(x, y));
    }

    pub fn set_gravity_multipier(&mut self, value: f64) {
        self.0.set_gravity_multipier(value);
    }

    pub fn set_restitution_multipier(&mut self, value: f64) {
        self.0.set_restitution_multipier(value);
    }

    pub fn set_friction_multipier(&mut self, value: f64) {
        self.0.set_friction_multipier(value);
    }

    pub fn set_static_friction(&mut self, enabled: bool) {
        self.0.set_static_friction(enabled);
    }

    pub fn set_dynamic_friction(&mut self, enabled: bool) {
        self.0.set_dynamic_friction(enabled);
    }
}
