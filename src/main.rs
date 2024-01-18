use glam::Vec3;
use crate::material::*;
use crate::object::*;
use crate::camera::Camera;

mod material;
mod ray;
mod interval;
mod object;
mod camera;
mod output;

fn main() {
    let mut camera = Camera::default();
    // Render
    let objects: Vec<Box<dyn Object>> = vec![
        Box::new(Sphere::new(Vec3::new(-0.5, 0.0, -1.0), 0.6, Diffuse::new(Vec3::new(0.5, 0.5, 0.5)))),
        Box::new(Sphere::new(Vec3::new(0.7, 0.3, -1.2), 0.5, Lambertian::new(Vec3::new(0.9, 0.4, 0.6)))),
    ];
    camera.render(&objects, output::Format::BMP).expect("Failed outputting image");
}
