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
        Box::new(Sphere::new(Vec3::new(0.0, -1000.5, -1.0), 1000.0, Diffuse::new(0.7, 0.7, 0.7))),
        Box::new(Sphere::new(Vec3::new(-0.5, 0.05, -1.0), 0.6, Diffuse::new(0.3, 0.0, 0.9))),
        Box::new(Sphere::new(Vec3::new(0.7, 0.0, -1.3), 0.5, Metal::new((0.7, 0.7, 0.7), 0.25))),
    ];
    camera.render(&objects, output::Format::BMP).expect("Failed outputting image");
}
