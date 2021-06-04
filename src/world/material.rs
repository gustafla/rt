use super::HitRecord;
use crate::Ray;
use rand::prelude::*;
use ultraviolet::Vec3;

pub trait Scatter<R: Rng>: Send + Sync {
    fn scatter(&self, rng: &mut R, r: Ray, hit: HitRecord) -> Option<(Vec3, Ray)>;
}

fn random_on_sphere(rng: &mut impl Rng) -> Vec3 {
    let phi = rng.gen_range(0f32..std::f32::consts::TAU);
    let z = rng.gen_range(-1f32..1.); // Equal to cos theta
    let sin_theta = (1. - z.powi(2)).sqrt();
    Vec3::new(sin_theta * phi.cos(), sin_theta * phi.sin(), z)
}

pub struct Lambertian {
    albedo: Vec3,
}

impl Lambertian {
    pub fn new(albedo: Vec3) -> Self {
        Self { albedo }
    }
}

impl<R: Rng> Scatter<R> for Lambertian {
    fn scatter(&self, rng: &mut R, r: Ray, hit: HitRecord) -> Option<(Vec3, Ray)> {
        let direction = hit.normal + random_on_sphere(rng);
        Some((self.albedo, Ray::new(hit.position, direction, r.time())))
    }
}

pub struct Metal {
    albedo: Vec3,
    fuzz: f32,
}

impl Metal {
    pub fn new(albedo: Vec3, fuzz: f32) -> Self {
        Self { albedo, fuzz }
    }
}

impl<R: Rng> Scatter<R> for Metal {
    fn scatter(&self, rng: &mut R, r: Ray, hit: HitRecord) -> Option<(Vec3, Ray)> {
        let direction = r.direction().reflected(hit.normal) + self.fuzz * random_on_sphere(rng);
        if direction.dot(hit.normal) > 0. {
            Some((self.albedo, Ray::new(hit.position, direction, r.time())))
        } else {
            None
        }
    }
}

fn reflectance(cos_theta: f32, refraction_ratio: f32) -> f32 {
    // Schlick's approximation
    let r0 = ((1. - refraction_ratio) / (1. + refraction_ratio)).powi(2);
    r0 + (1. - r0) * (1. - cos_theta).powi(5)
}

pub struct Dielectric {
    refraction: f32,
}

impl Dielectric {
    pub fn new(refraction: f32) -> Self {
        Self { refraction }
    }
}

impl<R: Rng> Scatter<R> for Dielectric {
    fn scatter(&self, rng: &mut R, r: Ray, hit: HitRecord) -> Option<(Vec3, Ray)> {
        let refraction_ratio = if hit.front_facing {
            1. / self.refraction
        } else {
            self.refraction
        };

        let cos_theta = (-r.direction()).dot(hit.normal);
        let sin_theta = (1. - cos_theta.powi(2)).sqrt();
        let reflectance = reflectance(cos_theta, refraction_ratio);

        let direction = if refraction_ratio * sin_theta > 1. || rng.gen::<f32>() < reflectance {
            r.direction().reflected(hit.normal)
        } else {
            r.direction().refracted(hit.normal, refraction_ratio)
        };

        Some((Vec3::one(), Ray::new(hit.position, direction, r.time())))
    }
}
