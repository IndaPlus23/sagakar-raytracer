use std::io::Error;
use rand::{thread_rng, rngs::ThreadRng, Rng};
use crate::output::{write_bmp, write_tga, Format};
use glam::Vec3;
use crate::ray::{Ray, Hit};
use crate::interval::Interval;
use crate::object::*;

type Color = Vec3;

pub struct Camera {
    image_width: u16,
    image_height: u16,
    center: Vec3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    viewport_pixel_origin: Vec3,
    image_data: Vec<Vec<u8>>,
    filename: String,
    rng: ThreadRng,
    pub samples: u32,
    max_depth: u32,
}

impl Camera {
    pub fn default() -> Camera {
        let image_width: u16 = 256;
        let image_height: u16 = 256;
        let viewport_height: f32 = 2.0;
        let viewport_width: f32 = viewport_height * (image_width as f32 / image_height as f32);
        let focal_length: f32 = 1.0;
        let center= Vec3::new(0.0, 0.0, 0.0);
        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, viewport_height, 0.0);
        let pixel_delta_u = viewport_u / image_width as f32;
        let pixel_delta_v = viewport_v / image_height as f32;
        let viewport_lower_left = center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        let viewport_pixel_origin = viewport_lower_left + (pixel_delta_u + pixel_delta_v) / 2.0;
        let mut image_data = vec![];
        image_data.resize(image_height as usize, vec![]);
        Camera {
            image_width,
            image_height,
            center,
            pixel_delta_u,
            pixel_delta_v,
            viewport_pixel_origin,
            image_data,
            filename: "output".to_owned(),
            rng: thread_rng(),
            samples: 10,
            max_depth: 15,
        }
    }

    pub fn render(&mut self, objects: &Vec<Box<dyn Object>>, format: Format) -> Result<(), Error> {
        // Scan left to right, bottom to top
        for image_y in 0..self.image_height {
            print!("\r{:3} lines remaining", self.image_height - image_y);
            for image_x in 0..self.image_width {
                // Sums to average the colors later
                let mut total_color = Color::new(0.0, 0.0, 0.0);
                for _i in 0..self.samples {
                    let ray = self.get_random_ray(image_x, image_y);
                    total_color += self.ray_to_color(&ray, &objects, self.max_depth);
                }
                // Average and add to image in LE order
                let average_color = gamma_correct(total_color / self.samples as f32);
                let bytes = color_to_bytes(average_color);
                self.image_data[image_y as usize].push(bytes.2);
                self.image_data[image_y as usize].push(bytes.1);
                self.image_data[image_y as usize].push(bytes.0);

            }
        }
        match format {
            Format::BMP => write_bmp(&self.image_data, &(self.filename.clone() + ".bmp")),
            Format::TGA => write_tga(&self.image_data, &(self.filename.clone() + ".tga"))
        }
    }

    fn get_random_ray(&mut self, image_x: u16, image_y: u16) -> Ray {
        let pixel_center = self.viewport_pixel_origin + image_x as f32 * self.pixel_delta_u + image_y as f32 * self.pixel_delta_v; 
        let sample_offset = (-0.5 + self.rng.gen::<f32>()) * self.pixel_delta_u + (-0.5 + self.rng.gen::<f32>()) * self.pixel_delta_u;
        let direction = pixel_center - self.center + sample_offset;
        return Ray::new(self.center, direction);
    }

    fn get_intersection(&mut self, ray: &Ray, objects: &Vec<Box<dyn Object>>, hit_interval: &Interval) -> Option<Hit> {
        let mut hit: Option<Hit> = None;
        let mut closest = hit_interval.max;
        for object in objects {
            if let Some(this_hit) = object.intersect(&mut self.rng, ray, &Interval::new(hit_interval.min, closest)) {
                closest = this_hit.t;
                hit = Some(this_hit);
            }
        }
        return hit;
    }

    fn ray_to_color(&mut self, ray: &Ray, objects: &Vec<Box<dyn Object>>, depth: u32) -> Color {
        if depth == 0 {
            return Color::new(0.0, 0.0, 0.0);
        }
        if let Some(hit) = self.get_intersection(ray, objects, &Interval::new(0.001, f32::MAX)) {
            let bounced_ray = hit.outgoing;
            let albedo = hit.albedo;
            let bounced = self.ray_to_color(&bounced_ray, objects, depth - 1);
            let final_color = Color::new(bounced.x * albedo.x, bounced.y * albedo.y, bounced.z * albedo.z);
            return final_color + hit.emitted;
        }
        return background_gradient(ray);
    }
}

/// Accepts a color in vector form and returns it as (red, green, blue) bytes
fn color_to_bytes(color: Color) -> (u8, u8, u8) {
    let color = color.clamp(Vec3::ZERO, Vec3::ONE);
    let red = lerp(0.0, 255.0, color.x) as u8;
    let green = lerp(0.0, 255.0, color.y) as u8;
    let blue = lerp(0.0, 255.0, color.z) as u8;
    return (red, green, blue);
}

fn background_gradient(ray: &Ray) -> Color {
    // let direction = ray.direction.normalize();
    // let t = direction.y;
    // let blue = lerp(155.0, 235.0, t) as u8;
    // let green = lerp(155.0, 206.0, t) as u8;
    // let red = lerp(155.0, 135.0, t) as u8;
    // return (blue, green, red);
    return Color::new(0.4, 0.4, 0.4);
}

fn gamma_correct(color: Color) -> Color {
    Color::new(color.x.sqrt(), color.y.sqrt(), color.z.sqrt())
}

// Linearly interpolates t ∈ [0, 1] to the range [v0, v1]
fn lerp(v0: f32, v1: f32, t: f32) -> f32 {
    (1.0 - t) * v0 + t * v1
}