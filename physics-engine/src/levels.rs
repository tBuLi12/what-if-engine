use crate::geometry::{Circle, Point};

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

pub struct Level {
    pub initial_ball_position: Point,
    pub circles: Vec<Entity<Circle>>,
    pub polygons: Vec<Entity<Vec<Point>>>,
    pub flags_positions: Vec<Point>,
}
