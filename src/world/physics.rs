use crate::Ray;
use std::ops::Range;
use ultraviolet::{Lerp, Vec3};

#[derive(Default)]
pub struct PhysicsFrame {
    pub position: Range<Vec3>,
}

impl PhysicsFrame {
    pub fn stationary(position: Vec3) -> Self {
        Self {
            position: position..position,
        }
    }

    pub fn position(&self, r: &Ray) -> Vec3 {
        self.position.start.lerp(self.position.end, r.time())
    }
}
