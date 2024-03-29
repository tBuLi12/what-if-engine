use std::{
    cell::RefCell,
    os::raw::c_void,
    rc::{Rc, Weak},
};

use rand::Rng;
use serde::{Deserialize, Serialize};
use tsify::Tsify;

use self::{
    binding::{Binding, Unbound},
    shape::{Circle, Collidable, Polygon},
};
use crate::{
    geometry::{self, Point, Vector},
    levels::Level,
};

mod binding;
pub mod compute;
pub mod shape;

const GRAVITY_COEFFICIENT: f64 = 0.00000981;
const MOVEMENT_COEFFICIENT: f64 = 0.00004;

#[derive(Serialize, Deserialize, Tsify, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct WithColor<S> {
    pub color: [f32; 3],
    pub shape: S,
}

impl<S> From<S> for WithColor<S> {
    fn from(shape: S) -> Self {
        let mut rng = rand::thread_rng();

        Self {
            color: [
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                rng.gen_range(0.0..1.0),
                // 0.0,
                // 0.0,
                // 0.0,
            ],
            shape,
        }
    }
}

#[derive(Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct DisplayMessage {
    pub polygons: Vec<WithColor<geometry::Polygon>>,
    pub circles: Vec<WithColor<geometry::Circle>>,
    pub flags: Vec<geometry::Polygon>,
    pub rigid_bindings: Vec<geometry::Point>,
    pub hinges: Vec<Point>,
    pub unbound_rigid_bindings: Vec<Point>,
    pub unbound_hinges: Vec<Point>,
}

fn to_geometry<G>(
    shapes: &mut Vec<WithColor<Weak<RefCell<impl Into<G> + Clone>>>>,
) -> Vec<WithColor<G>> {
    let mut geometry_shapes = Vec::with_capacity(shapes.len());
    shapes.retain(|colored_shape| {
        if let Some(shape) = colored_shape.shape.upgrade() {
            geometry_shapes.push(WithColor {
                color: colored_shape.color,
                shape: shape.borrow().clone().into(),
            });
            true
        } else {
            false
        }
    });
    geometry_shapes
}

#[cfg(test)]
macro_rules! make_shape {
    ($(($x:expr, $y:expr)),*$(,)?) => {
        $crate::physics::shape::Polygon::new(vec![
            $($crate::geometry::Point($x, $y)),*
        ])
    };
}

#[cfg(test)]
pub(crate) use make_shape;

struct EntityCfg {
    is_erasable: bool,
    is_bindable: bool,
    is_static: bool,
}

impl Default for EntityCfg {
    fn default() -> Self {
        EntityCfg {
            is_erasable: true,
            is_bindable: true,
            is_static: false,
        }
    }
}

struct Entity {
    bindings: Vec<(Binding, Weak<RefCell<dyn Collidable>>)>,
    unbound: Vec<Unbound>,
    is_erasable: bool,
    is_bindable: bool,
    is_static: bool,
    shape: Rc<RefCell<dyn Collidable>>,
}

impl Entity {
    fn new(shape: Rc<RefCell<dyn Collidable>>, entity_type: EntityCfg) -> Self {
        let EntityCfg {
            is_erasable,
            is_bindable,
            is_static,
        } = entity_type;

        Self {
            bindings: vec![],
            unbound: vec![],
            shape,
            is_static,
            is_erasable,
            is_bindable,
        }
    }

    fn add_rigid(&mut self, at: Point) {
        self.unbound
            .push(Unbound::new_rigid(&*self.shape.borrow(), at))
    }

    fn add_hinge(&mut self, at: Point) {
        self.unbound
            .push(Unbound::new_hinge(&*self.shape.borrow(), at))
    }

    fn try_bind(&mut self, target: &Rc<RefCell<dyn Collidable>>) {
        self.unbound.retain(|unbound| {
            if let Some(binding) =
                Binding::try_bind(&*self.shape.borrow_mut(), *unbound, &*target.borrow_mut())
            {
                self.bindings.push((binding, Rc::downgrade(target)));
                false
            } else {
                true
            }
        })
    }
}

pub struct Engine {
    // each entity may contain bidings with pointers to entities
    // ocurring later in the vector
    entities: Vec<Entity>,
    // circles and polygons kept separate on the side,
    // because that's how they need to be passed to the graphics.
    // The Rc<RefCell<_>> is pretty much unavoidable,
    // mostly because shapes need to be accessed both via the main vector of entities
    // as well as through bindings. If bindings stored indexes into the vector rather than
    // weak pointers then they would have to be manually updated after removing an entity
    polygons: Vec<WithColor<Weak<RefCell<Polygon>>>>,
    circles: Vec<WithColor<Weak<RefCell<Circle>>>>,
    main_ball_starting_position: Point,
    flags: Vec<Polygon>,
    restitution_mulipiler: f64,
    friction_mulipiler: f64,
    gravity_mulipiler: f64,
    static_friction_enabled: bool,
    dynamic_friction_enabled: bool,
}

impl Engine {
    pub fn new(
        Level {
            initial_ball_position,
            circles,
            polygons,
            flags_positions,
        }: Level,
    ) -> Self {
        let n_of_circles = circles.len() + 1;
        let n_of_polygons = polygons.len();

        let mut engine = Self {
            entities: Vec::with_capacity(n_of_circles + n_of_polygons),
            circles: Vec::with_capacity(n_of_circles),
            polygons: Vec::with_capacity(n_of_polygons),
            main_ball_starting_position: initial_ball_position,
            flags: flags_positions
                .into_iter()
                .map(|Point(x, y)| {
                    Polygon::new(vec![
                        geometry::Point(x, y),
                        geometry::Point(x + 0.1, y),
                        geometry::Point(x + 0.1, y + 0.1),
                        geometry::Point(x, y + 0.1),
                    ])
                })
                .collect(),
            friction_mulipiler: 1.0,
            restitution_mulipiler: 1.0,
            gravity_mulipiler: 1.0,
            dynamic_friction_enabled: true,
            static_friction_enabled: true,
        };

        let main_ball_weak = engine.add_entity(
            Circle::new(initial_ball_position, 0.1),
            EntityCfg {
                is_bindable: true,
                is_erasable: false,
                is_static: false,
            },
        );
        engine.circles.push(main_ball_weak.into());

        for entity in polygons {
            let weak = engine.add_entity(
                Polygon::new(entity.shape),
                EntityCfg {
                    is_bindable: entity.is_bindable,
                    is_static: entity.is_static,
                    is_erasable: false,
                },
            );
            engine.polygons.push(weak.into())
        }

        for entity in circles {
            let geometry::Circle { center, radius } = entity.shape;
            let weak = engine.add_entity(
                Circle::new(center, radius),
                EntityCfg {
                    is_bindable: entity.is_bindable,
                    is_static: entity.is_static,
                    is_erasable: false,
                },
            );
            engine.circles.push(weak.into())
        }

        engine.prune_and_send_shapes();
        engine
    }

    pub fn run_iteration(&mut self, microseconds: f64) -> DisplayMessage {
        // move all shapes, removing ones out of bounds
        // don't remove the first one though, as it's the main ball
        let mut is_main_ball = true;
        self.entities.retain_mut(|entity| {
            let mut shape = entity.shape.borrow_mut();

            if !entity.is_static {
                shape.update_position(microseconds, self.gravity_mulipiler);
            }

            let retain = shape.collision_data_mut().centroid.1 > -5.0 || is_main_ball;
            is_main_ball = false;
            retain
        });

        // return main ball to starting point if out of bounds
        // and check win condition
        {
            let mut ball = self.entities[0].shape.borrow_mut();
            let data = ball.collision_data_mut();

            if data.centroid.0.abs() > 5.0 || data.centroid.1 < -5.0 {
                data.centroid = self.main_ball_starting_position;
                data.angular_velocity = 0.0;
                data.velocity = Vector::ZERO;
            }

            self.flags
                .retain(|flag| compute::collision(&*ball, flag).is_none());

            // if self.flags.is_empty() {
            //     alert("you win");
            // }
        }

        // iterate over all pairs of shapes
        {
            let mut i = 0;
            while let [this, rest @ ..] = &mut self.entities[i..] {
                let mut shape = this.shape.borrow_mut();

                // collide them if they are not bound
                rest.iter_mut().for_each(|other| {
                    let mut is_boud_to_other = false;
                    this.bindings.retain(|(_, target)| {
                        let valid = target.strong_count() > 0;
                        if valid {
                            is_boud_to_other = is_boud_to_other
                                || std::ptr::eq(
                                    target.as_ptr() as *const c_void,
                                    (&*other.shape) as *const _ as *const c_void,
                                )
                        }
                        valid
                    });

                    if !is_boud_to_other {
                        shape.collide(
                            &mut *other.shape.borrow_mut(),
                            microseconds,
                            self.restitution_mulipiler,
                            self.friction_mulipiler,
                            self.static_friction_enabled,
                            self.dynamic_friction_enabled,
                        )
                    }
                });

                // enforce binding constraints
                this.bindings.iter().for_each(|(binding, target)| {
                    if let Some(other) = target.upgrade() {
                        binding.enforce(
                            &mut *shape,
                            &mut *other.borrow_mut(),
                            microseconds,
                            self.restitution_mulipiler,
                            self.friction_mulipiler,
                            self.static_friction_enabled,
                            self.dynamic_friction_enabled,
                        )
                    }
                });

                i += 1;
            }
        }

        self.prune_and_send_shapes()
    }

    fn prune_and_send_shapes(&mut self) -> DisplayMessage {
        let mut rigid_bindings = Vec::new();
        let mut hinges = Vec::new();
        let mut unbound_rigid_bindings = Vec::new();
        let mut unbound_hinges = Vec::new();

        for Entity {
            bindings,
            unbound,
            shape,
            ..
        } in &self.entities
        {
            for (binding, _) in bindings {
                match binding {
                    Binding::Hinge { first, .. } => hinges.push(first.on(&*shape.borrow())),
                    Binding::Rigid {
                        first: (p1, p2), ..
                    } => {
                        let shape = shape.borrow();
                        rigid_bindings.push((p1.on(&*shape) + p2.on(&*shape)) * 0.5)
                    }
                }
            }

            for binding in unbound {
                match binding {
                    Unbound::Hinge(point) => unbound_hinges.push(point.on(&*shape.borrow())),
                    Unbound::Rigid(point) => {
                        unbound_rigid_bindings.push(point.on(&*shape.borrow()))
                    }
                }
            }
        }

        DisplayMessage {
            polygons: to_geometry(&mut self.polygons),
            circles: to_geometry(&mut self.circles),
            flags: self.flags.iter().cloned().map(Into::into).collect(),
            rigid_bindings,
            hinges,
            unbound_rigid_bindings,
            unbound_hinges,
        }
    }

    pub fn try_bind(&mut self, new_shape: &Rc<RefCell<dyn Collidable>>) {
        self.entities
            .iter_mut()
            .for_each(|shape| shape.try_bind(new_shape))
    }

    fn add_entity<S: Collidable + 'static>(
        &mut self,
        mut shape: S,
        entity_cfg: EntityCfg,
    ) -> Weak<RefCell<S>> {
        if entity_cfg.is_static {
            shape.collision_data_mut().mass = f64::INFINITY;
            shape.collision_data_mut().inertia = f64::INFINITY;
        }

        let shape = Rc::new(RefCell::new(shape));
        let shape_weak = Rc::downgrade(&shape);
        let shape_dyn: Rc<RefCell<dyn Collidable>> = shape;

        self.try_bind(&shape_dyn);
        self.entities.push(Entity::new(shape_dyn, entity_cfg));
        shape_weak
    }

    pub fn add_circle(&mut self, circle: Circle) {
        let weak_circle = self.add_entity(circle, EntityCfg::default());
        self.circles.push(weak_circle.into());
    }

    pub fn add_polygon(&mut self, polygon: Polygon) {
        let weak_polygon = self.add_entity(polygon, EntityCfg::default());
        self.polygons.push(weak_polygon.into());
    }

    pub fn erase_at(&mut self, point: Point) {
        if let Some(i) = self
            .entities
            .iter()
            .position(|shape| shape.shape.borrow().includes(point))
        {
            if self.entities[i].is_erasable {
                self.entities.remove(i);
            }
        }
    }

    pub fn add_hinge(&mut self, point: Point) {
        if let Some(i) = self
            .entities
            .iter()
            .position(|shape| shape.shape.borrow().includes(point) && shape.is_bindable)
        {
            self.entities[i].add_hinge(point);
        }
    }

    pub fn add_rigid(&mut self, point: Point) {
        if let Some(i) = self
            .entities
            .iter()
            .position(|shape| shape.shape.borrow().includes(point) && shape.is_bindable)
        {
            self.entities[i].add_rigid(point);
        }
    }

    pub fn set_gravity_multipier(&mut self, value: f64) {
        self.gravity_mulipiler = value;
    }

    pub fn set_restitution_multipier(&mut self, value: f64) {
        self.restitution_mulipiler = value;
    }

    pub fn set_friction_multipier(&mut self, value: f64) {
        self.friction_mulipiler = value;
    }

    pub fn set_static_friction(&mut self, enabled: bool) {
        self.static_friction_enabled = enabled;
    }

    pub fn set_dynamic_friction(&mut self, enabled: bool) {
        self.dynamic_friction_enabled = enabled;
    }
}
