use std::{io::Error, clone};
use rand::{thread_rng, rngs::ThreadRng, Rng};
use output::{write_bmp, write_tga};
use glam::Vec3;
use dyn_clone::DynClone;

enum Face {
    Front,
    Back
}

enum Format {
    BMP,
    TGA
}


trait Material: DynClone {
    // Get the bounced ray direction given an incoming ray and an outward normal
    fn bounce_direction(&self, rng: &mut ThreadRng, incoming: &Ray, normal: Vec3) -> Vec3;
    // Get the proportion of bounced blue, green and red light
    fn brg_reflectivity(&self) -> (f32, f32, f32);
}

trait Object {
    // If ray intersects, return point of intersection
    // Else return none
    fn intersect(&self, ray: &Ray, hit_interval: &Interval) -> Option<Hit>;
    // Return the unit normal at the given point
    fn normal(&self, point: Vec3) -> Vec3;
    // Return the material of the object
    fn get_material(&self) -> Box<dyn Material>;
}


// Contains information on a ray-object intersection
struct Hit {
    t: f32,
    position: Vec3,
    normal: Vec3,
    face: Face,
    material: Box<dyn Material>
}

impl Hit {
    fn new(ray: &Ray, t: f32, position: Vec3, outward_normal: Vec3, material: Box<dyn Material>) -> Hit {
        let face = match outward_normal.dot(ray.direction) < 0.0 {
            true => Face::Front,
            false => Face::Back,
        };
        let normal = match face {
            Face::Front => outward_normal,
            Face::Back => -outward_normal
        };
        Hit {
            t,
            position,
            normal,
            face,
            material
        }
    }
}

struct Interval {
    min: f32,
    max: f32
}

impl Interval {
    fn new(min: f32, max: f32) -> Interval {
        Interval {
            min,
            max
        }
    }

    fn contains(&self, num: f32) -> bool {
        num >= self.min && num <= self.max 
    }

    fn surrounds(&self, num: f32) -> bool {
        num > self.min && num < self.max
    }
}

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    // Creates a new ray of length 0
    fn new(origin: Vec3, direction: Vec3) -> Ray {
        Ray {
            origin,
            direction,
        }
    }

    // Returns the current location of the ray
    fn pos(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

    fn closest_intersection(&self, objects: &Vec<Box<dyn Object>>, hit_interval: &Interval) -> Option<Hit> {
        let mut hit: Option<Hit> = None;
        let mut closest = hit_interval.max;
        for object in objects {
            if let Some(this_hit) = object.intersect(self, &Interval::new(hit_interval.min, closest)) {
                closest = this_hit.t;
                hit = Some(this_hit);
            }
        }
        return hit;
    }
}

struct Sphere<T: Material> {
    center: Vec3,
    radius: f32,
    material: Box<T>
}

impl<T: Material + 'static> Object for Sphere<T> {
    // Returns None if no hit, otherwise returns the t value at intersection
    // I used the pq formula for this, because the american formula is like math uncanny valley
    // 
    fn intersect(&self, ray: &Ray, hit_interval: &Interval) -> Option<Hit> {
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
        return Some(Hit::new(ray, t, position, self.normal(position), self.get_material()))
    }

    fn normal(&self, point: Vec3) -> Vec3 {
        (point - self.center).normalize()
    }

    fn get_material(&self) -> Box<dyn Material> {
        dyn_clone::clone_box(&*self.material)
    }
}

impl<T: Material> Sphere<T>{
    fn new(center: Vec3, radius: f32, material: T) -> Sphere<T> {
        Sphere {
            center,
            radius,
            material: Box::new(material)
        }
    }
}

#[derive(Clone)]
struct Diffuse {
    color: Vec3,
}

impl Material for Diffuse {
    fn bounce_direction(&self, rng: &mut ThreadRng, incoming: &Ray, normal: Vec3) -> Vec3 {
        random_on_hemisphere(rng, &normal)
    }

    fn brg_reflectivity(&self) -> (f32, f32, f32) {
        (self.color.x, self.color.y, self.color.z)
    }
}

impl Diffuse {
    fn new(color: Vec3) -> Diffuse {
        Diffuse {color}
    }
}

#[derive(Clone)]
struct Lambertian {
    color: Vec3
}

impl Material for Lambertian {
    fn bounce_direction(&self, rng: &mut ThreadRng, incoming: &Ray, normal: Vec3) -> Vec3 {
        normal + random_unit_vector(rng)
    }

    fn brg_reflectivity(&self) -> (f32, f32, f32) {
        (self.color.x, self.color.y, self.color.z)
    }
}

impl Lambertian {
    fn new(color: Vec3) -> Lambertian {
        Lambertian{color}
    }
}

struct Camera {
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
    fn default() -> Camera {
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

    fn render(&mut self, objects: &Vec<Box<dyn Object>>, format: Format) -> Result<(), Error> {
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
                self.image_data[image_y as usize].push((blue_sum / self.samples) as u8);
                self.image_data[image_y as usize].push((green_sum / self.samples) as u8);
                self.image_data[image_y as usize].push((red_sum / self.samples) as u8);

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

    fn ray_to_color(&mut self, ray: &Ray, objects: &Vec<Box<dyn Object>>, depth: u32) -> (u8, u8, u8) {
        if depth == 0 {
            return (0, 0, 0);
        }
        if let Some(hit) = ray.closest_intersection(objects, &Interval::new(0.001, f32::MAX)) {
            let material = hit.material;
            let direction = material.bounce_direction(&mut self.rng, ray, hit.normal);
            let (reflect_blue, reflect_green, reflect_red) = material.brg_reflectivity();
            let (bounced_blue, bounced_green, bounced_red) = self.ray_to_color(&Ray::new(hit.position, direction), objects, depth - 1);
            let final_blue = (bounced_blue as f32 * reflect_blue) as u8;
            let final_green = (bounced_green as f32 * reflect_green) as u8;
            let final_red = (bounced_red as f32 * reflect_red) as u8;
            return (final_blue, final_green, final_red)
        }
        return background_gradient(ray);
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

fn background_gradient(ray: &Ray) -> (u8, u8, u8) {
    let direction = ray.direction.normalize();
    let t = direction.y;
    let blue = lerp(139.0, 255.0, t) as u8;
    let green = lerp(0.0, 255.0, t) as u8;
    let red = lerp(0.0, 255.0, t) as u8;
    return (blue, green, red);
}

fn rand_in_interval(rng: &mut ThreadRng, interval: &Interval) -> f32 {
    interval.min + (interval.max - interval.min) * rng.gen::<f32>()
}

// Linearly interpolates t ∈ [0, 1] to the range [v0, v1]
fn lerp(v0: f32, v1: f32, t: f32) -> f32 {
    (1.0 - t) * v0 + t * v1
}

fn main() {
    let mut camera = Camera::default();
    // Render
    let objects: Vec<Box<dyn Object>> = vec![
        Box::new(Sphere::new(Vec3::new(-0.5, 0.0, -1.0), 0.6, Diffuse::new(Vec3::new(0.5, 0.5, 0.5)))),
        Box::new(Sphere::new(Vec3::new(0.7, 0.3, -1.2), 0.5, Lambertian::new(Vec3::new(0.9, 0.4, 0.6)))),
    ];
    camera.render(&objects, Format::BMP).expect("Failed outputting image");
}

mod output {
    use std::{
        fs::File,
        io::{Error, Write},
    };
    // -- TGA parameters --
    // All values little-endian
    const TGA_DEFAULT_HEADER: [u8; 18] = [
        0x00, // ID length: no ID field
        0x00, // Color map type: no color map
        0x02, // Image type: uncompressed true color
        0x00, 0x00, 0x00, 0x00, 0x00, // Irrelevant color map stuff
        0x00, 0x00, 0x00, 0x00, // (x, y) origin (should be 0)
        0x00, 0x00, // Width in pixels (we want to change this)
        0x00, 0x00, // Height in pixels (and this)
        0x18, // Bits per pixel: 24
        0x00, // Random stuff we don't care about
    ];
    const TGA_WIDTH_INDEX: usize = 12;
    const TGA_HEIGHT_INDEX: usize = 14;

    // -- BMP parameters --
    // Standard bitmap header followed by BITMAPCOREHEADER
    const BMP_DEFAULT_HEADER: [u8; 26] = [
        'B' as u8, 'M' as u8, // BMP identifier
        0x00, 0x00, 0x00, 0x00, // Filesize (we want to change this)
        0x00, 0x00, 0x00, 0x00, // Reserved (can be ignored)
        0x1A, 0x00, 0x00, 0x00, // Image data offset (size of header)
        0x0C, 0x00, 0x00, 0x00, // Header size: 12 bytes
        0x00, 0x00, // Width in pixels (we want to change this)
        0x00, 0x00, // Height in pixels (we want to change this too!)
        0x01, 0x00, // Number of color planes, whatever that is (must be 1 anyway)
        0x18, 0x00, // Bits per pixel: 24
    ];
    const BMP_FILESIZE_INDEX: usize = 2;
    const BMP_WIDTH_INDEX: usize = 18;
    const BMP_HEIGHT_INDEX: usize = 20;

    // Output the generated image to a .tga file
    pub fn write_tga(
        image_data: &[Vec<u8>],
        filename: &str,
    ) -> Result<(), Error> {
        let mut header = TGA_DEFAULT_HEADER.clone().to_vec();
        let height = image_data.len() as u16;
        let width = (image_data[0].len() / 3) as u16;
        // Put dimensions in the header
        header.splice(TGA_WIDTH_INDEX..TGA_WIDTH_INDEX + 2, width.to_le_bytes());
        header.splice(TGA_HEIGHT_INDEX..TGA_HEIGHT_INDEX + 2, height.to_le_bytes());
        // Create and write the file
        let mut output_file = File::create(filename)?;
        output_file.write(&header)?;
        for row in image_data {
            output_file.write(row)?;
        }
        Ok(())
    }

    // Output the generated image to a .bmp file
    pub fn write_bmp(
        image_data: &[Vec<u8>],
        filename: &str,
    ) -> Result<(), Error> {
        let mut header = BMP_DEFAULT_HEADER.clone().to_vec();
        let height = image_data.len() as u16;
        let width = (image_data[0].len() / 3) as u16;
        header.splice(BMP_WIDTH_INDEX..BMP_WIDTH_INDEX + 2, width.to_le_bytes());
        header.splice(BMP_HEIGHT_INDEX..BMP_HEIGHT_INDEX + 2, height.to_le_bytes());
        let padding: Vec<u8> = vec![0; ((width * 3) % 4) as usize]; // The length of every row of image data must be a multiple of 4
        // Do the padding (i was tired when writing this)
        let image_data = image_data
            .to_owned()
            .into_iter()
            .map(|mut row| {
                row.extend_from_slice(&padding);
                row
            })
            .flatten()
            .collect::<Vec<u8>>();
        let filesize = (header.len() + image_data.len()) as u32;
        // Put filesize and dimensions in the header
        header.splice(
            BMP_FILESIZE_INDEX..BMP_FILESIZE_INDEX + 4,
            filesize.to_le_bytes(),
        );
        let mut output_file = File::create(filename)?;
        output_file.write(&header)?;
        output_file.write(&image_data)?;
        Ok(())
    }
}
