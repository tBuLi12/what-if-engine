use std::{panic::RefUnwindSafe, time::Duration};

use crate::{
    geometry::{Point, Vector},
    physics::compute,
};

use super::{
    binding::PointOnShape, compute::simplex::Vertex, GRAVITY_COEFFICIENT, MOVEMENT_COEFFICIENT,
};

mod circle;
mod polygon;

pub use circle::Circle;
pub use polygon::Polygon;

pub trait Bounded {
    fn support_vector(&self, direction: Vector) -> Point;
    fn includes(&self, point: Point) -> bool;
}

pub trait Collidable: Bounded + RefUnwindSafe {
    fn rotate(&mut self, angle: f64);
    fn translate(&mut self, translation: Vector);
    fn collision_data_mut(&mut self) -> &mut CollisionData;

    fn resolve_collision_with(
        &mut self,
        other: &mut dyn Collidable,
        collision: Vertex,
        microseconds: f64,
        restitution_mulipiler: f64,
        friction_mulipiler: f64,
        static_friction_enabled: bool,
        dynamic_friction_enabled: bool,
    ) {
        const RESTITUTION: f64 = 0.2;
        let restitution = restitution_mulipiler * RESTITUTION;

        let first = self.collision_data_mut();
        let second = other.collision_data_mut();

        let first_offset = first.centroid.to(collision.created_from.0);
        let second_offset = second.centroid.to(collision.created_from.1);
        let normal = collision.point.unit();
        let first_velocity =
            first.velocity - (first_offset * first.angular_velocity).perpendicular();
        let second_velocity =
            second.velocity - (second_offset * second.angular_velocity).perpendicular();
        let relative_velocity = second_velocity - first_velocity;

        let impulse = compute::impulse(
            first.clone(),
            second.clone(),
            first_offset,
            second_offset,
            normal,
            relative_velocity,
            restitution + 1.0,
        );

        if impulse > 0.0 {
            let friction_normal = -normal.perpendicular();

            let static_friction_impulse = compute::impulse(
                first.clone(),
                second.clone(),
                first_offset,
                second_offset,
                friction_normal,
                relative_velocity,
                1.0,
            );

            let friction_impulse = if static_friction_impulse > impulse * friction_mulipiler * 1e-4
            {
                if dynamic_friction_enabled {
                    compute::impulse(
                        first.clone(),
                        second.clone(),
                        first_offset,
                        second_offset,
                        friction_normal,
                        relative_velocity,
                        (50.0 * collision.point.norm() * friction_mulipiler).min(1.0),
                    )
                } else {
                    0.0
                }
            } else {
                if static_friction_enabled {
                    static_friction_impulse
                } else {
                    0.0
                }
            };

            first.velocity -= normal * (impulse / first.mass);
            first.angular_velocity -= impulse * first_offset.cross(normal) / first.inertia;

            second.velocity += normal * (impulse / second.mass);
            second.angular_velocity += impulse * second_offset.cross(normal) / second.inertia;

            first.velocity -= friction_normal * (friction_impulse / first.mass);
            first.angular_velocity -=
                friction_impulse * first_offset.cross(friction_normal) / first.inertia;

            second.velocity += friction_normal * (friction_impulse / second.mass);
            second.angular_velocity +=
                friction_impulse * second_offset.cross(friction_normal) / second.inertia;
        }

        if first.mass.is_finite() || second.mass.is_finite() {
            let translation = normal * collision.point.norm().min(1e-6 * microseconds);
            let i1 = first.mass.recip();
            let i2 = second.mass.recip();
            let i_sum = i1 + i2;

            self.translate(-translation * (i1 / i_sum));
            other.translate(translation * (i2 / i_sum));
        }
    }

    fn collide(
        &mut self,
        other: &mut dyn Collidable,
        microseconds: f64,
        restitution_mulipiler: f64,
        friction_mulipiler: f64,
        static_friction_enabled: bool,
        dynamic_friction_enabled: bool,
    ) {
        let Some(collision) = compute::collision(self, other) else {
            return;
        };

        if collision.point.is_close_enough_to(Vector::ZERO) {
            return;
        }

        self.resolve_collision_with(
            other,
            collision,
            microseconds,
            restitution_mulipiler,
            friction_mulipiler,
            static_friction_enabled,
            dynamic_friction_enabled,
        );
    }

    fn resolve_point_reference(&self, point_ref: PointOnShape) -> Point;
    fn create_point_reference(&self, point: Point) -> PointOnShape;

    fn update_position(&mut self, microseconds: f64, gravity_multiplier: f64) {
        let velocity = self.collision_data_mut().velocity;
        let angular_velocity = self.collision_data_mut().angular_velocity;

        self.collision_data_mut().velocity +=
            Point(0.0, gravity_multiplier * GRAVITY_COEFFICIENT * microseconds);
        self.rotate(angular_velocity * MOVEMENT_COEFFICIENT * microseconds);
        self.translate(velocity * MOVEMENT_COEFFICIENT * microseconds);
    }
}

pub trait Shape: Collidable + Clone + Into<Self::Underlying> {
    type Underlying;
}

#[derive(Clone, Debug)]
pub struct CollisionData {
    pub centroid: Point,
    pub mass: f64,
    pub inertia: f64,
    pub velocity: Vector,
    pub angular_velocity: f64,
}
