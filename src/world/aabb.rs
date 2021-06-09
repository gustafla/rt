use crate::Ray;
use std::ops::Range;
use ultraviolet::Vec3;

pub struct Aabb(Range<Vec3>);

impl Aabb {
    pub fn new(range: Range<Vec3>) -> Self {
        Self(range)
    }

    pub fn surrounding(ranges: Range<Range<Vec3>>) -> Self {
        Self(
            ranges.start.start.min_by_component(ranges.end.start)
                ..ranges.start.end.max_by_component(ranges.end.end),
        )
    }

    pub fn hit(&self, ray: &Ray, t_range: Range<f32>) -> bool {
        let mut t_min = t_range.start;
        let mut t_max = t_range.end;

        for a in 0..3 {
            // Extract vector components
            let inv_direction = 1. / ray.direction().as_slice()[a];
            let start = self.0.start.as_slice()[a];
            let end = self.0.end.as_slice()[a];
            let origin = ray.origin().as_slice()[a];

            // Compute t-intevals for each component of ray and AABB vectors
            let mut t0 = (start - origin) * inv_direction;
            let mut t1 = (end - origin) * inv_direction;

            // Sort
            if inv_direction < 0. {
                std::mem::swap(&mut t0, &mut t1);
            }
            t_min = t0.max(t_min);
            t_max = t1.min(t_max);

            if t_max <= t_min {
                return false; // No overlap in t-interval
            }
        }

        true
    }
}
