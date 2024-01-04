use crate::geometry::{Circle, Point};
use serde::{Deserialize, Serialize};
use tsify::Tsify;

#[derive(Serialize, Deserialize, Tsify, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Entity<S> {
    pub shape: S,
    pub is_static: bool,
    pub is_bindable: bool,
}

/// Represents a single level
///
/// intended to be loadaed from a file specified by the user in RON notation
/// and passed directly to the physics engine
///

#[derive(Serialize, Deserialize, Tsify, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Level {
    pub initial_ball_position: Point,
    pub circles: Vec<Entity<Circle>>,
    pub polygons: Vec<Entity<Vec<Point>>>,
    pub flags_positions: Vec<Point>,
}
