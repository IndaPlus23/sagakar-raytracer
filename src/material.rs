use rand::{rngs::ThreadRng, Rng};
use glam::Vec3;
use crate::ray::Ray;
use crate::interval::Interval;

type Color = Vec3;

pub trait Material {
    // Get the bounced ray direction given an incoming ray and an outward normal
    fn bounce(&self, rng: &mut ThreadRng, incoming: &Ray, position: Vec3, normal: Vec3) -> Ray;
    // Get the proportion of bounced blue, green and red light
    fn albedo(&self) -> Color;
    // Does the material emit light?
    fn is_emitter(&self) -> bool {
        false
    }
    // If it is, what light does it emit?
    fn emit(&self) -> Color {
        Color::ZERO
    }
}

pub struct Diffuse {
    color: Color,
}

impl Material for Diffuse {
    fn bounce(&self, rng: &mut ThreadRng, _incoming: &Ray, position: Vec3, normal: Vec3) -> Ray {
        let direction = random_on_hemisphere(rng, &normal);
        return Ray::new(position, direction);
    }

    fn albedo(&self) -> Color {
        self.color
    }
}

impl Diffuse {
    pub fn new(red: f32, green: f32, blue: f32) -> Diffuse {
        Diffuse{color: Color::new(red, green, blue)}
    }
}

pub struct Lambertian {
    color: Color
}

impl Material for Lambertian {
    fn bounce(&self, rng: &mut ThreadRng, _incoming: &Ray, position: Vec3, normal: Vec3) -> Ray {
        // We risk creating a near-zero vector, in which case it's normalized
        let direction = normalize_if_tiny(normal + random_unit_vector(rng));
        return Ray::new(position, direction);
    }

    fn albedo(&self) -> Color {
        self.color
    }
}

impl Lambertian {
    pub fn new(red: f32, green: f32, blue: f32) -> Lambertian {
        Lambertian{color: Color::new(red, green, blue)}
    }
}

pub struct Metal {
    color: Color,
    fuzz: f32
}

impl Material for Metal {
    fn bounce(&self, rng: &mut ThreadRng, incoming: &Ray, position: Vec3, normal: Vec3) -> Ray {
        let direction = reflect(incoming.direction, normal);
        let fuzzed_direction = normalize_if_tiny(direction + random_unit_vector(rng) * self.fuzz);
        return Ray::new(position, fuzzed_direction);
    }

    fn albedo(&self) -> Color {
        self.color
    }
}

impl Metal {
    pub fn new(color: Color, fuzz: f32) -> Metal {
        Metal{
            color,
            fuzz
        }
    }
}

pub struct DiffuseLight {
    light: Color
}

impl Material for DiffuseLight {
    fn albedo(&self) -> Color {
        Vec3::new(1.0, 1.0, 1.0)
    }

    fn bounce(&self, rng: &mut ThreadRng, incoming: &Ray, position: Vec3, normal: Vec3) -> Ray {
        let direction = random_on_hemisphere(rng, &normal);
        return Ray::new(position, direction);
    }

    fn is_emitter(&self) -> bool {
        true
    }

    fn emit(&self) -> Color {
        self.light
    }
}

impl DiffuseLight {
    pub fn new(red: f32, green: f32, blue: f32) -> DiffuseLight {
        DiffuseLight{light: Color::new(red, green, blue)}
    }
}

fn random_unit_vector(rng: &mut ThreadRng) -> Vec3 {
    Vec3::new(rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0), rng.gen_range(-1.0..=1.0)).normalize()
}

fn random_on_hemisphere(rng: &mut ThreadRng, normal: &Vec3) -> Vec3 {
    let vector = random_unit_vector(rng);
    match vector.dot(*normal) > 0.0 {
        true => vector,
        false => -vector
    }
}

fn reflect(incoming: Vec3, normal: Vec3) -> Vec3 {
    // Since the incoming vector is not normalized, scale the normal to use in reflection
    let scaled_normal = -(incoming.dot(normal) * normal); 
    return incoming + 2.0 * scaled_normal;
}

/// If a vector is very close to 0, normalize to avoid funny errors
fn normalize_if_tiny(vec: Vec3) -> Vec3 {
    let interval = Interval::new(-0.000001, 0.000001);
    if interval.contains(vec.x) && interval.contains(vec.y) && interval.contains(vec.z) {
        return vec.normalize();
    }
    return vec;
}