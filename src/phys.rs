use raylib::prelude::*;

pub trait Collides<T> {
    fn interact(&self, other: &T) -> Option<Vector2>;
}

pub struct Body {
    pub velocity: Vector2,
    pub position: Vector2,
}

pub struct Ground {
    pub body: Body,
    pub rotation: f32,
}

pub struct Circle {
    pub body: Body,
    pub radius: f32,
}

impl Collides<Circle> for Circle {
    fn interact(&self, other: &Circle) -> Option<Vector2> {
        let min_dist = self.radius + other.radius;
        let interaction_vec = self.body.position - other.body.position;
        let intersection_len = interaction_vec.length() - min_dist;

        if intersection_len > f32::EPSILON {
            None
        } else {
            Some(-interaction_vec.normalized() * intersection_len)
        }
    }
}

impl Collides<Ground> for Circle {
    fn interact(&self, other: &Ground) -> Option<Vector2> {
        let d = Vector2::from(other.rotation.sin_cos());
        let v = self.body.position - other.body.position;
        let p = d * v.dot(d);

        let intersection = p.length() - self.radius;

        if intersection > f32::EPSILON {
            None
        } else {
            Some(p.normalized() * -intersection)
        }
    }
}