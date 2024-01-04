use levels::Level;
use physics::{compute, shape, WithColor};
use wasm_bindgen::prelude::*;

mod geometry;
mod levels;
mod physics;

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Circle {
    pub radius: f64,
    pub center: Point,
    pub is_static: bool,
    pub is_bindable: bool,
}

#[wasm_bindgen]
impl Circle {
    #[wasm_bindgen(constructor)]
    pub fn new(radius: f64, center: Point, is_static: bool, is_bindable: bool) -> Circle {
        Circle {
            radius,
            center,
            is_static,
            is_bindable,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct Polygon {
    pub points: Vec<Point>,
    pub is_static: bool,
    pub is_bindable: bool,
}

#[wasm_bindgen]
impl Polygon {
    #[wasm_bindgen(constructor)]
    pub fn new(points: Vec<Point>, is_static: bool, is_bindable: bool) -> Polygon {
        Polygon {
            points,
            is_bindable,
            is_static,
        }
    }
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct Init {
    pub ball_position: Point,
    pub flags: Vec<Point>,
    pub circles: Vec<Circle>,
    pub polygons: Vec<Polygon>,
}

#[wasm_bindgen]
impl Init {
    #[wasm_bindgen(constructor)]
    pub fn new(
        ball_position: Point,
        flags: Vec<Point>,
        circles: Vec<Circle>,
        polygons: Vec<Polygon>,
    ) -> Init {
        Init {
            ball_position,
            flags,
            circles,
            polygons,
        }
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Point(pub f64, pub f64);

#[wasm_bindgen]
impl Point {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64) -> Point {
        Point(x, y)
    }
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct Color {
    pub r: f64,
    pub g: f64,
    pub b: f64,
}

#[wasm_bindgen]
#[derive(Clone, Copy)]
pub struct ColoredCircle {
    pub radius: f64,
    pub center: Point,
    pub color: Color,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct ColoredPolygon {
    pub points: Vec<Point>,
    pub color: Color,
}

#[wasm_bindgen(getter_with_clone)]
#[derive(Clone)]
pub struct Flag {
    pub points: Vec<Point>,
}

#[wasm_bindgen(getter_with_clone, inspectable)]
#[derive(Clone)]
pub struct Shapes {
    pub polygons: Vec<ColoredPolygon>,
    pub circles: Vec<ColoredCircle>,
    pub flags: Vec<Flag>,
    pub rigid_bindings: Vec<Point>,
    pub hinges: Vec<Point>,
    pub unbound_rigid_bindings: Vec<Point>,
    pub unbound_hinges: Vec<Point>,
}

#[wasm_bindgen]
pub struct Engine(Box<physics::Engine>);

trait IntoJs {
    type Js;
    fn into_js(self) -> Self::Js;
}

impl IntoJs for geometry::Point {
    type Js = Point;

    fn into_js(self) -> Self::Js {
        Point(self.0, self.1)
    }
}

impl<T: IntoJs> IntoJs for Vec<T> {
    type Js = Vec<T::Js>;
    fn into_js(self) -> Self::Js {
        self.into_iter().map(IntoJs::into_js).collect()
    }
}

impl IntoJs for WithColor<geometry::Circle> {
    type Js = ColoredCircle;

    fn into_js(self) -> Self::Js {
        ColoredCircle {
            center: self.shape.center.into_js(),
            color: self.color.into_js(),
            radius: self.shape.radius,
        }
    }
}

impl IntoJs for WithColor<geometry::Polygon> {
    type Js = ColoredPolygon;

    fn into_js(self) -> Self::Js {
        ColoredPolygon {
            points: self.shape.into_js(),
            color: self.color.into_js(),
        }
    }
}

impl IntoJs for geometry::Polygon {
    type Js = Vec<Point>;

    fn into_js(self) -> Self::Js {
        self.vertices.into_js()
    }
}

impl IntoJs for [f32; 3] {
    type Js = Color;

    fn into_js(self) -> Self::Js {
        Color {
            r: self[0] as f64,
            g: self[1] as f64,
            b: self[2] as f64,
        }
    }
}

#[wasm_bindgen]
impl Engine {
    pub fn create(init: Init) -> Self {
        Engine(Box::new(physics::Engine::new(Level {
            circles: init
                .circles
                .into_iter()
                .map(|circle| levels::Entity {
                    shape: geometry::Circle {
                        center: geometry::Point(circle.center.0, circle.center.1),
                        radius: circle.radius,
                    },
                    is_bindable: circle.is_bindable,
                    is_static: circle.is_static,
                })
                .collect(),
            flags_positions: init
                .flags
                .into_iter()
                .map(|Point(x, y)| geometry::Point(x, y))
                .collect(),
            initial_ball_position: geometry::Point(init.ball_position.0, init.ball_position.1),
            polygons: init
                .polygons
                .into_iter()
                .map(|poly| levels::Entity {
                    shape: poly
                        .points
                        .into_iter()
                        .map(|Point(x, y)| geometry::Point(x, y))
                        .collect(),
                    is_bindable: poly.is_bindable,
                    is_static: poly.is_bindable,
                })
                .collect(),
        })))
    }

    pub fn run_iteration(&mut self) -> Shapes {
        let msg = self.0.run_iteration();
        Shapes {
            circles: msg.circles.into_js(),
            flags: msg
                .flags
                .into_iter()
                .map(|flag| Flag {
                    points: flag.into_js(),
                })
                .collect(),
            hinges: msg.hinges.into_js(),
            rigid_bindings: msg.rigid_bindings.into_js(),
            unbound_hinges: msg.unbound_hinges.into_js(),
            unbound_rigid_bindings: msg.unbound_rigid_bindings.into_js(),
            polygons: msg.polygons.into_js(),
        }
    }

    pub fn add_circle(&mut self, x: f64, y: f64, radius: f64) {
        self.0
            .add_circle(shape::Circle::new(geometry::Point(x, y), radius))
    }

    pub fn add_polygon(&mut self, points: Vec<Point>) {
        self.0.add_polygon(compute::hull::<24>(
            points
                .into_iter()
                .map(|Point(x, y)| geometry::Point(x as f64, -y as f64)),
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
}
