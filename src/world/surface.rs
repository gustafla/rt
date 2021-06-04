use super::PhysicsFrame;
use crate::Ray;
use std::ops::Range;
use ultraviolet::Vec3;

pub struct HitRecord {
    pub position: Vec3,
    pub normal: Vec3,
    pub t: f32,
    pub front_facing: bool,
}

impl HitRecord {
    pub fn new(position: Vec3, outward_normal: Vec3, t: f32, r: &Ray) -> Self {
        let front_facing = r.direction().dot(outward_normal) < 0.;
        Self {
            position,
            normal: if front_facing {
                outward_normal
            } else {
                -outward_normal
            },
            t,
            front_facing,
        }
    }
}

pub trait Hit: Send + Sync {
    fn hit(&self, r: &Ray, t_range: Range<f32>, physics: &PhysicsFrame) -> Option<HitRecord>;
}

pub struct Sphere {
    radius: f32,
}

impl Sphere {
    pub fn new(radius: f32) -> Self {
        Self { radius }
    }
}

impl Hit for Sphere {
    fn hit(&self, r: &Ray, t_range: Range<f32>, physics: &PhysicsFrame) -> Option<HitRecord> {
        let center = physics.position(r);
        let oc = r.origin() - center;
        let a = r.direction().mag().powi(2);
        let half_b = oc.dot(r.direction());
        let c = oc.mag().powi(2) - self.radius.powi(2);

        let discriminant = half_b.powi(2) - a * c;
        if discriminant < 0. {
            return None;
        }

        // Find the nearest root that lies in the acceptable range
        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;
        if root < t_range.start || t_range.end < root {
            root = (-half_b + sqrtd) / a;
            if root < t_range.start || t_range.end < root {
                return None;
            }
        }

        let position = r.at(root);
        let outward_normal = (position - center) / self.radius;
        Some(HitRecord::new(position, outward_normal, root, r))
    }
}
