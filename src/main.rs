use std::env;
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
    let args = env::args().collect::<Vec<String>>();

    if args.len() > 1 {
        let samples: u32 = args[1].parse().expect("Invalid number of samples");
        camera.samples = samples;
    }
    
    // Create a cornell box
    let mut scene: Vec<Box<dyn Object>> = vec![
        // Floor
        Box::new(Rect::new(
            Vec3::new(-1.0, -1.0, -0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -2.0),
            Lambertian::new(0.85, 0.85, 0.85)
        )),
        // Ceiling
        Box::new(Rect::new(
            Vec3::new(-1.0, 1.0, -2.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 2.0),
            Lambertian::new(0.85, 0.85, 0.85)
        )),
        //Left wall
        Box::new(Rect::new(
            Vec3::new(-1.0, -1.0, 0.0),
            Vec3::new(0.0, 0.0, -2.0),
            Vec3::new(0.0, 2.0, 0.0),
            Lambertian::new(0.85, 0.0, 0.0)
        )),
        //Right wall
        Box::new(Rect::new(
            Vec3::new(1.0, -1.0, -2.0),
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::new(0.0, 2.0, 0.0),
            Lambertian::new(0.0, 0.85, 0.0)
        )),
        // Back wall
        Box::new(Rect::new(
            Vec3::new(-1.0, -1.0, -2.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Diffuse::new(0.85, 0.85, 0.85)
        )),
        // Light
        Box::new(Rect::new(
            Vec3::new(-0.25, 0.99, -1.75),
            Vec3::new(0.5, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 0.5),
            DiffuseLight::new(7.0, 7.0, 7.0)
        )),
    ];

    let mut objects: Vec<Box<dyn Object>> = vec![
        Box::new(Sphere::new(Vec3::new(-0.5, -0.5, -1.2), 0.5, Lambertian::new(0.9, 0.2, 0.9))),
        Box::new(Sphere::new(Vec3::new(0.4, -0.6, -1.5), 0.4, Metal::new(Vec3::new(1.0, 1.0, 1.0), 0.03))),
    ];

    scene.append(&mut objects);
    camera.render(&scene, output::Format::BMP).expect("Failed outputting image");
}
