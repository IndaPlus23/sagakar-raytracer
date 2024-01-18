use glam::Vec3;

use crate::interval::Interval;

pub struct BoundingBox {
    x: Interval,
    y: Interval,
    z: Interval
}

impl BoundingBox {
    pub fn new(x: Interval, y: Interval, z: Interval) -> BoundingBox {
        return BoundingBox {
            x,
            y,
            z
        }
    }

    pub fn from_corners(c0: Vec3, c1: Vec3) -> BoundingBox {
        return BoundingBox {
            x: Interval::new(c0.x.min(c1.x), c0.x.max(c1.x)),
            y: Interval::new(c0.y.min(c1.y), c0.y.max(c1.y)),
            z: Interval::new(c0.z.min(c1.z), c0.z.max(c1.z))
        };
    }
}