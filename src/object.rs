use glam::Vec3;
use rand::rngs::ThreadRng;
use crate::ray::{Ray, Hit};
use crate::interval::Interval;
use crate::material::Material;


pub trait Object {
    // If ray intersects, return point of intersection
    // Else return none
    fn intersect(&self, rng: &mut ThreadRng, ray: &Ray, hit_interval: &Interval) -> Option<Hit>;
    // Return the unit normal at the given point
    fn normal(&self, point: Vec3) -> Vec3;
    // Return the material of the object
    fn bounce(&self, rng: &mut ThreadRng, incoming: &Ray, position: Vec3, normal: Vec3) -> Ray;
    // Return the blue, green and red albedos of the object
    fn albedo(&self) -> (f32, f32, f32);
    fn is_emitter(&self) -> bool;
    fn emit(&self) -> (u8, u8, u8);
}

pub struct Sphere<T: Material> {
    center: Vec3,
    radius: f32,
    material: T
}

impl<T: Material> Object for Sphere<T> {
    // Returns None if no hit, otherwise returns the t value at intersection
    // I used the pq formula for this, because the american formula is like math uncanny valley
    // 
    fn intersect(&self, rng: &mut ThreadRng, ray: &Ray, hit_interval: &Interval) -> Option<Hit> {
        let center_to_origin = ray.origin - self.center;
        let half_p = ray.direction.dot(center_to_origin) / ray.direction.length_squared();
        let q = (center_to_origin.length_squared() - self.radius.powi(2)) / ray.direction.length_squared();
        let discriminant = half_p.powi(2) - q;
        // If no real roots, return None
        if discriminant < 0.0 {
            return None;
        }
        let mut t = -half_p - discriminant.sqrt();
        // If the negative root is not in the t interval, flip the root
        if !hit_interval.surrounds(t) {
            t = -half_p + discriminant.sqrt();
            // If it's still not in the interval, it's too far away. Return None
            if !hit_interval.surrounds(t) {
                return None;
            }
        }
        let position = ray.pos(t);
        let normal = self.normal(position);
        let outgoing = self.bounce(rng, ray, position, normal);
        return Some(Hit::new(
            ray,
            t,
            position,
            normal,
            self.albedo(),
            outgoing,
            self.is_emitter(),
            self.emit()
        ));
    }

    fn normal(&self, point: Vec3) -> Vec3 {
        (point - self.center).normalize()
    }

    fn bounce(&self, rng: &mut ThreadRng, incoming: &Ray, position: Vec3, normal: Vec3) -> Ray {
        self.material.bounce(rng, incoming, position, normal)
    }

    fn albedo(&self) -> (f32, f32, f32) {
        self.material.albedo()
    }

    fn is_emitter(&self) -> bool {
        self.material.is_emitter()
    }

    fn emit(&self) -> (u8, u8, u8) {
        self.material.emit()
    }
}

impl<T: Material> Sphere<T>{
    pub fn new(center: Vec3, radius: f32, material: T) -> Sphere<T> {
        Sphere {
            center,
            radius,
            material: material
        }
    }
}

/// A rectangle defined by an origin point and two vectors
pub struct Rect<T: Material> {
    origin: Vec3, // lower left with positive x in U and positive y in V
    u: Vec3,
    v: Vec3,
    normal: Vec3,
    // Let (A, B, C) be the normal vector of the plane the quad lies on
    // D is then derived from Ax + By + Cz = D
    // There really is no good descriptive name for it
    d: f32, 
    material: T
}

impl <T: Material> Object for Rect<T> {
    fn intersect(&self, rng: &mut ThreadRng, ray: &Ray, hit_interval: &Interval) -> Option<Hit> {
        let dividend = self.d - self.normal.dot(ray.origin);
        let divisor = self.normal.dot(ray.direction);
        // If ray is near parallel, return None
        if divisor < 0.000001 {
            return  None;
        }
        let t = dividend / divisor;
        // If t is outside the hit interval, return None
        if !hit_interval.surrounds(t) {
            return None;
        }
        let position = ray.pos(t);
        let local_position = position - self.origin;
        let alpha = local_position.dot(self.u) / self.u.length().powi(2);
        let beta = local_position.dot(self.v) / self.v.length().powi(2);
        let coordinate_range = Interval::new(0.0, 1.0);
        if !coordinate_range.contains(alpha) || !coordinate_range.contains(beta) {
            return None;
        }
        return Some(Hit::new(
            ray,
            t,
            position,
            self.normal,
            self.albedo(),
            self.bounce(rng, ray, position, self.normal),
            self.is_emitter(),
            self.emit()
        ));
    }

    fn normal(&self, _point: Vec3) -> Vec3 {
        self.normal
    }

    fn albedo(&self) -> (f32, f32, f32) {
        self.material.albedo()
    }

    fn bounce(&self, rng: &mut ThreadRng, incoming: &Ray, position: Vec3, normal: Vec3) -> Ray {
        self.material.bounce(rng, incoming, position, normal)
    }

    fn is_emitter(&self) -> bool {
        self.material.is_emitter()
    }

    fn emit(&self) -> (u8, u8, u8) {
        self.material.emit()
    }
}

impl <T: Material> Rect<T> {
    pub fn new(origin: Vec3, u: Vec3, v: Vec3, material: T) -> Rect<T> {
        let normal = v.cross(u).normalize();
        // Recall the equation for D
        // Knowing that the origin must be on the correct plane, D is simply the dot product of the origin and the normal
        let d = normal.dot(origin);
        return Rect{
            origin,
            u,
            v,
            normal,
            d,
            material
        }
    }
}