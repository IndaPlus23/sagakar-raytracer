use output::{write_bmp, write_tga};
use glam::Vec3;

const OUTPUT_FORMAT: Format = Format::BMP;

enum Format {
    BMP,
    TGA
}

trait Object {
    // If ray intersects, return point of intersection
    // Else return none
    fn intersect(&self, ray: &Ray) -> Option<Hit>;
    // Return the unit normal at the given point
    fn normal(&self, point: Vec3) -> Vec3;
}

// Contains information on a ray-object intersection
struct Hit {
    t: f32,
    position: Vec3,
    normal: Vec3
}

impl Hit {
    fn new(t: f32, position: Vec3, normal: Vec3) -> Hit {
        Hit {
            t,
            position,
            normal
        }
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
}

struct Sphere {
    center: Vec3,
    radius: f32
}

impl Object for Sphere {
    // Returns None if no hit, otherwise returns the t value at intersection
    // I used the pq formula for this, because the american formula is like math uncanny valley
    fn intersect(&self, ray: &Ray) -> Option<Hit> {
        let center_to_origin = ray.origin - self.center;
        let half_p = ray.direction.dot(center_to_origin) / ray.direction.length_squared();
        let q = (center_to_origin.length_squared() - self.radius.powi(2)) / ray.direction.length_squared();
        let discriminant = half_p.powi(2) - q;
        if discriminant < 0.0 {
            return None;
        }
        let t = -half_p - discriminant.sqrt();
        let position = ray.pos(t);
        return Some(Hit::new(t, position, self.normal(position)))
    }

    fn normal(&self, point: Vec3) -> Vec3 {
        (point - self.center).normalize()
    }
}

impl Sphere {
    fn new(center: Vec3, radius: f32) -> Sphere {
        Sphere {
            center,
            radius
        }
    }
}

fn main() {
    // Image dimenstsions
    let image_width: u16 = 320;
    let image_height: u16 = 240;
    // Camera properties
    let viewport_height: f32 = 2.0;
    let viewport_width: f32 = viewport_height * (image_width as f32 / image_height as f32);
    let focal_length: f32 = 1.0;
    let camera_center= Vec3::new(0.0, 0.0, 0.0);
    // Viewport coordinate system
    let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
    let viewport_v = Vec3::new(0.0, viewport_height, 0.0);
    let pixel_delta_u = viewport_u / image_width as f32;
    let pixel_delta_v = viewport_v / image_height as f32;
    let viewport_lower_left = camera_center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
    let viewport_pixel_origin = viewport_lower_left + (pixel_delta_u + pixel_delta_v) / 2.0; // Pixel origin is inset by half a pixel width
    
    // Render
    let objects: Vec<Box<dyn Object>> = vec![
        Box::new(Sphere::new(Vec3::new(-0.5, 0.0, -1.0), 0.6)),
        Box::new(Sphere::new(Vec3::new(0.7, 0.3, -1.2), 0.5))
    ];

    let mut image_data: Vec<Vec<u8>> = vec![]; // Color data, stored per row with the origin in the bottom left
    image_data.resize(image_height as usize, vec![]);
    // Scan left to right, bottom to top
    for image_y in 0..image_height {
        print!("\r{} lines remaining", image_height - image_y);
        for image_x in 0..image_width {
            let pixel_center = viewport_pixel_origin + image_x as f32 * pixel_delta_u + image_y as f32 * pixel_delta_v; 
            let direction = pixel_center - camera_center;
            let ray = Ray::new(camera_center, direction);
            
            let pixel_color = ray_to_color(&ray, &objects);
            image_data[image_y as usize].extend_from_slice(&pixel_color);
        }
    }
    
    
    write_bmp(&image_data, "output.bmp").unwrap();
    write_tga(&image_data, "output.tga").unwrap();
}

fn ray_to_color(ray: &Ray, objects: &Vec<Box<dyn Object>>) -> [u8;3] {
    for object in objects {
        if let Some(hit) = object.intersect(ray) {
            let normal = hit.normal;
            let blue = lerp(55.0, 255.0, normal.x) as u8;
            let green = lerp(55.0, 255.0, normal.y) as u8;
            let red = lerp(55.0, 255.0, normal.z) as u8;
            return [blue, green, red];
        }
    };
    let direction = ray.direction.normalize();
    let t = direction.y;
    let blue = lerp(139.0, 255.0, t) as u8;
    let green = lerp(0.0, 255.0, t) as u8;
    let red = lerp(0.0, 255.0, t) as u8;
    return [blue, green, red];
}

// Linearly interpolates t ∈ [0, 1] to the range [v0, v1]
fn lerp(v0: f32, v1: f32, t: f32) -> f32 {
    (1.0 - t) * v0 + t * v1
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
