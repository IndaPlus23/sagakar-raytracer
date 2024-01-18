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
    
    // Create a cornell box
    let mut scene: Vec<Box<dyn Object>> = vec![
        // Floor
        Box::new(Rect::new(
            Vec3::new(-1.0, -1.0, -0.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, -2.0),
            Lambertian::new(1.0, 1.0, 1.0)
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
            Vec3::new(-1.0, -1.0, -0.0),
            Vec3::new(0.0, 0.0, -2.0),
            Vec3::new(0.0, 2.0, 0.0),
            Lambertian::new(0.0, 0.0, 1.0)
        )),
        //Right wall
        Box::new(Rect::new(
            Vec3::new(1.0, -1.0, -2.0),
            Vec3::new(0.0, 0.0, 2.0),
            Vec3::new(0.0, 2.0, 0.0),
            Lambertian::new(0.0, 1.0, 0.0)
        )),
        // Back wall
        Box::new(Rect::new(
            Vec3::new(-1.0, -1.0, -2.0),
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(0.0, 2.0, 0.0),
            Lambertian::new(0.85, 0.85, 0.85)
        )),
        Box::new(Rect::new(
            Vec3::new(-0.5, 0.99, -1.5),
            Vec3::new(1.0, 0.0, 0.0),
            Vec3::new(0.0, 0.0, 1.0),
            DiffuseLight::new((255, 255, 255))
        )),
    ];

    let mut objects: Vec<Box<dyn Object>> = vec![
        Box::new(Sphere::new(Vec3::new(-0.5, 0.1, -1.3), 0.6, Metal::new((0.9, 0.3, 0.9), 0.05))),
        Box::new(Sphere::new(Vec3::new(0.7, 0.0, -1.3), 0.5, Metal::new((0.7, 0.7, 0.7), 0.05))),
    ];

    scene.append(&mut objects);
    camera.render(&scene, output::Format::BMP).expect("Failed outputting image");
}
