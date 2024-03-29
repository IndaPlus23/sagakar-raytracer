use glam::Vec3;

type Color = Vec3;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    // Creates a new ray of length 0
    pub fn new(origin: Vec3, direction: Vec3) -> Ray {
        Ray {
            origin,
            direction,
        }
    }

    // Returns the current location of the ray
    pub fn pos(&self, t: f32) -> Vec3 {
        self.origin + self.direction * t
    }

}

// Contains information on a ray-object intersection
pub struct Hit {
    pub t: f32,
    pub position: Vec3,
    pub normal: Vec3,
    pub front_face: bool,
    pub albedo: Color,
    pub outgoing: Ray,
    pub is_emitter: bool,
    pub emitted: Color
}

impl Hit {
    pub fn new(
        ray: &Ray,
        t: f32,
        position: Vec3,
        outward_normal: Vec3,
        albedo: Color,
        outgoing: Ray,
        is_emitter: bool,
        emitted: Color
    ) -> Hit {
        let front_face = outward_normal.dot(ray.direction) < 0.0;
        let normal = match front_face {
            true => outward_normal,
            false => -outward_normal
        };
        Hit {
            t,
            position,
            normal,
            front_face,
            albedo,
            outgoing,
            is_emitter,
            emitted
        }
    }
}