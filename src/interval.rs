pub struct Interval {
    pub min: f32,
    pub max: f32
}

impl Interval {
    pub fn new(min: f32, max: f32) -> Interval {
        Interval {
            min,
            max
        }
    }

    pub fn contains(&self, num: f32) -> bool {
        num >= self.min && num <= self.max 
    }

    pub fn surrounds(&self, num: f32) -> bool {
        num > self.min && num < self.max
    }
}