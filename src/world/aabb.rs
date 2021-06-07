use crate::Ray;
use std::ops::Range;
use ultraviolet::Vec3;

pub struct AABB(Range<Vec3>);

impl AABB {
    fn new(range: Range<Vec3>) -> Self {
        Self(range)
    }

    fn hit(&self, ray: &Ray, t_range: Range<f32>) -> bool {
        for a in 0..3 {
            let inv_direction = 1. / ray.direction().as_slice()[a];
            let start = self.0.start.as_slice()[a];
            let end = self.0.end.as_slice()[a];
            let origin = ray.origin().as_slice()[a];

            let mut t0 = (start - origin) * inv_direction;
            let mut t1 = (end - origin) * inv_direction;
            if inv_direction < 0. {
                std::mem::swap(&mut t0, &mut t1);
            }

            if t1.min(t_range.end) <= t0.max(t_range.start) {
                return false;
            }
        }
        true
    }
}
