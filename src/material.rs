use rand::{rngs::ThreadRng, Rng};
use glam::Vec3;
use crate::ray::Ray;

pub trait Material {
    // Get the bounced ray direction given an incoming ray and an outward normal
    fn bounce(&self, rng: &mut ThreadRng, incoming: &Ray, position: Vec3, normal: Vec3) -> Ray;
    // Get the proportion of bounced blue, green and red light
    fn albedo(&self) -> (f32, f32, f32);
}

pub struct Diffuse {
    color: Vec3,
}

impl Material for Diffuse {
    fn bounce(&self, rng: &mut ThreadRng, _incoming: &Ray, position: Vec3, normal: Vec3) -> Ray {
        let direction = random_on_hemisphere(rng, &normal);
        return Ray::new(position, direction);
    }

    fn albedo(&self) -> (f32, f32, f32) {
        (self.color.x, self.color.y, self.color.z)
    }
}

impl Diffuse {
    pub fn new(color: Vec3) -> Diffuse {
        Diffuse {color}
    }
}

pub struct Lambertian {
    color: Vec3
}

impl Material for Lambertian {
    fn bounce(&self, rng: &mut ThreadRng, _incoming: &Ray, position: Vec3, normal: Vec3) -> Ray {
        let direction = normal + random_unit_vector(rng);
        return Ray::new(position, direction);
    }

    fn albedo(&self) -> (f32, f32, f32) {
        (self.color.x, self.color.y, self.color.z)
    }
}

impl Lambertian {
    pub fn new(color: Vec3) -> Lambertian {
        Lambertian{color}
    }
}

fn random_unit_vector(rng: &mut ThreadRng) -> Vec3 {
    Vec3::new(rng.gen(), rng.gen(), rng.gen()).normalize()
}

fn random_on_hemisphere(rng: &mut ThreadRng, normal: &Vec3) -> Vec3 {
    let vector = random_unit_vector(rng);
    match vector.dot(*normal) > 0.0 {
        true => vector,
        false => -vector
    }
}