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

    pub fn position(&self, time: f32) -> Vec3 {
        self.position.start.lerp(self.position.end, time)
    }
}
