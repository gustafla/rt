pub mod material;
pub mod surface;

use crate::Ray;
use material::Material;
use surface::{HitRecord, Surface};

pub struct Object {
    pub surface: Surface,
    pub material: Material,
}

pub struct World {
    objects: Vec<Object>,
}

impl World {
    pub fn new(objects: Vec<Object>) -> Self {
        Self { objects }
    }

    pub fn traverse(&self, r: &Ray, t_min: f32) -> Option<(HitRecord, &Material)> {
        let mut nearest_hit = None;
        let mut nearest_t = f32::INFINITY;

        for Object { surface, material } in self.objects.iter() {
            if let Some(hit) = surface.hit(r, t_min..nearest_t) {
                nearest_t = hit.t;
                nearest_hit = Some((hit, material));
            }
        }

        nearest_hit
    }
}
