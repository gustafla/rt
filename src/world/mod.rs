pub mod material;
pub mod physics;
pub mod surface;

use crate::Ray;
use material::{Dielectric, Lambertian, Metal, Scatter};
use physics::PhysicsFrame;
use rand::prelude::*;
use surface::{Hit, HitRecord, Sphere};
use ultraviolet::{Lerp, Vec3};

pub struct Object<R: Rng> {
    pub surface: Box<dyn Hit>,
    pub material: Box<dyn Scatter<R>>,
    pub physics: PhysicsFrame,
}

pub struct World<R: Rng> {
    objects: Vec<Object<R>>,
}

impl<R: Rng> World<R> {
    pub fn new(objects: Vec<Object<R>>) -> Self {
        Self { objects }
    }

    pub fn random(rng: &mut impl Rng) -> Self {
        let mut objects = vec![Object {
            surface: Box::new(Sphere::new(1000.)),
            material: Box::new(Lambertian::new(Vec3::one() * 0.5)),
            physics: PhysicsFrame::stationary(Vec3::new(0., -1000., 0.)),
        }];

        for a in -11..=11 {
            for b in -11..=11 {
                let center = Vec3::new(
                    a as f32 + rng.gen_range(0f32..0.9),
                    0.2,
                    b as f32 + rng.gen_range(0f32..0.9),
                );

                let (velocity, material) = match rng.gen_range(0..=100) {
                    // Diffuse
                    0..=79 => (
                        Vec3::unit_y() * rng.gen_range(0f32..0.5),
                        Box::new(Lambertian::new(
                            Vec3::from(rng.gen::<[f32; 3]>()) * Vec3::from(rng.gen::<[f32; 3]>()),
                        )) as Box<dyn Scatter<R>>,
                    ),
                    // Metal
                    80..=94 => (
                        Vec3::zero(),
                        Box::new(Metal::new(
                            Vec3::from(rng.gen::<[f32; 3]>()).lerp(Vec3::one(), 0.4),
                            rng.gen_range(0.0..0.2),
                        )) as Box<dyn Scatter<R>>,
                    ),
                    // Glass
                    _ => (
                        Vec3::zero(),
                        Box::new(Dielectric::new(1.5)) as Box<dyn Scatter<R>>,
                    ),
                };

                objects.push(Object {
                    surface: Box::new(Sphere::new(0.2)),
                    material,
                    physics: PhysicsFrame {
                        position: center..center + velocity,
                    },
                });
            }
        }

        objects.extend(vec![
            Object {
                surface: Box::new(Sphere::new(1.)),
                material: Box::new(Dielectric::new(1.5)),
                physics: PhysicsFrame::stationary(Vec3::new(0., 1., 0.)),
            },
            Object {
                surface: Box::new(Sphere::new(1.)),
                material: Box::new(Lambertian::new(Vec3::new(0.4, 0.2, 0.1))),
                physics: PhysicsFrame::stationary(Vec3::new(-4., 1., 0.)),
            },
            Object {
                surface: Box::new(Sphere::new(1.)),
                material: Box::new(Metal::new(Vec3::new(0.7, 0.6, 0.5), 0.)),
                physics: PhysicsFrame::stationary(Vec3::new(4., 1., 0.)),
            },
        ]);

        Self::new(objects)
    }

    pub fn traverse(&self, r: &Ray, t_min: f32) -> Option<(HitRecord, &dyn Scatter<R>)> {
        let mut nearest_hit = None;
        let mut nearest_t = f32::INFINITY;

        for Object {
            surface,
            material,
            physics,
        } in &self.objects
        {
            if let Some(hit) = surface.hit(r, t_min..nearest_t, physics) {
                nearest_t = hit.t;
                nearest_hit = Some((hit, material.as_ref()));
            }
        }

        nearest_hit
    }
}
