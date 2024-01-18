use std::io::Error;
use rand::{thread_rng, rngs::ThreadRng, Rng};
use crate::output::{write_bmp, write_tga, Format};
use glam::Vec3;
use crate::ray::{Ray, Hit};
use crate::interval::Interval;
use crate::object::*;

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
    samples: u32,
    max_depth: u32,
}

impl Camera {
    pub fn default() -> Camera {
        let image_width: u16 = 320;
        let image_height: u16 = 240;
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
            samples: 100,
            max_depth: 30,
        }
    }

    pub fn render(&mut self, objects: &Vec<Box<dyn Object>>, format: Format) -> Result<(), Error> {
        // Scan left to right, bottom to top
        for image_y in 0..self.image_height {
            print!("\r{:3} lines remaining", self.image_height - image_y);
            for image_x in 0..self.image_width {
                // Sums to average the colors later
                let (mut blue_sum, mut green_sum, mut red_sum) = (0u32, 0u32, 0u32);
                for _i in 0..self.samples {
                    let ray = self.get_random_ray(image_x, image_y);
                    let (blue, green, red) = self.ray_to_color(&ray, &objects, self.max_depth);
                    blue_sum += blue as u32;
                    green_sum += green as u32;
                    red_sum += red as u32;
                }
                // Average and add to image in LE order
                let blue = (blue_sum / self.samples) as u8;
                let green = (green_sum / self.samples) as u8;
                let red = (red_sum / self.samples) as u8;
                self.image_data[image_y as usize].push(blue);
                self.image_data[image_y as usize].push(green);
                self.image_data[image_y as usize].push(red);

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

    fn ray_to_color(&mut self, ray: &Ray, objects: &Vec<Box<dyn Object>>, depth: u32) -> (u8, u8, u8) {
        if depth == 0 {
            return (0, 0, 0);
        }
        if let Some(hit) = self.get_intersection(ray, objects, &Interval::new(0.001, f32::MAX)) {
            let bounced_ray = hit.outgoing;
            let (reflect_blue, reflect_green, reflect_red) = hit.albedo;
            let (bounced_blue, bounced_green, bounced_red) = self.ray_to_color(&bounced_ray, objects, depth - 1);
            let final_blue = (bounced_blue as f32 * reflect_blue) as u8;
            let final_green = (bounced_green as f32 * reflect_green) as u8;
            let final_red = (bounced_red as f32 * reflect_red) as u8;
            return (final_blue, final_green, final_red)
        }
        return background_gradient(ray);
    }
}

fn background_gradient(ray: &Ray) -> (u8, u8, u8) {
    let direction = ray.direction.normalize();
    let t = direction.y;
    let blue = lerp(139.0, 255.0, t) as u8;
    let green = lerp(0.0, 255.0, t) as u8;
    let red = lerp(0.0, 255.0, t) as u8;
    return (blue, green, red);
}

// Linearly interpolates t ∈ [0, 1] to the range [v0, v1]
fn lerp(v0: f32, v1: f32, t: f32) -> f32 {
    (1.0 - t) * v0 + t * v1
}