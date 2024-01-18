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
}

pub struct Sphere<T: Material> {
    center: Vec3,
    radius: f32,
    material: Box<T>
}

impl<T: Material + 'static> Object for Sphere<T> {
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
        return Some(Hit::new(ray, t, position, normal, self.albedo(), outgoing));
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
}

impl<T: Material> Sphere<T>{
    pub fn new(center: Vec3, radius: f32, material: T) -> Sphere<T> {
        Sphere {
            center,
            radius,
            material: Box::new(material)
        }
    }
}