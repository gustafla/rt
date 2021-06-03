use super::HitRecord;
use crate::Ray;
use glam::Vec3;
use rand::prelude::*;
use rand_xorshift::XorShiftRng;

pub trait Scatter {
    fn scatter(&self, rng: &mut XorShiftRng, r: &Ray, hit: &HitRecord) -> Option<(Vec3, Ray)>;
}

fn random_in_sphere(rng: &mut XorShiftRng) -> Vec3 {
    loop {
        let v = Vec3::from(rng.gen::<[f32; 3]>()) * 2. - Vec3::ONE;
        let len = v.length();
        if len < 1. && len > 0.0001 {
            return v;
        }
    }
}

pub struct Lambertian {
    albedo: Vec3,
}

impl Lambertian {
    pub fn new(albedo: Vec3) -> Self {
        Self { albedo }
    }
}

impl Scatter for Lambertian {
    fn scatter(&self, rng: &mut XorShiftRng, _: &Ray, hit: &HitRecord) -> Option<(Vec3, Ray)> {
        let mut direction = hit.normal + random_in_sphere(rng).normalize();
        if direction.length_squared() < 0.001 {
            direction = hit.normal;
        }
        Some((self.albedo, Ray::new(hit.position, direction)))
    }
}

fn reflect(v: Vec3, normal: Vec3) -> Vec3 {
    v - 2. * v.dot(normal) * normal
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

impl Scatter for Metal {
    fn scatter(&self, rng: &mut XorShiftRng, r: &Ray, hit: &HitRecord) -> Option<(Vec3, Ray)> {
        let reflected = reflect(r.direction(), hit.normal).normalize();
        let scattered = reflected + self.fuzz * random_in_sphere(rng);
        if scattered.dot(hit.normal) > 0. {
            Some((self.albedo, Ray::new(hit.position, scattered)))
        } else {
            None
        }
    }
}

fn refract(v: Vec3, normal: Vec3, refraction_ratio: f32) -> Vec3 {
    let cos_theta = (-v).dot(normal);
    let perpendicular = refraction_ratio * (v + cos_theta * normal);
    let parallel = -(1. - perpendicular.length_squared()).abs().sqrt() * normal;
    perpendicular + parallel
}

pub struct Dielectric {
    refraction: f32,
}

impl Dielectric {
    pub fn new(refraction: f32) -> Self {
        Self { refraction }
    }

    fn reflectance(cos_theta: f32, refraction_ratio: f32) -> f32 {
        // Schlick's approximation
        let r0 = ((1. - refraction_ratio) / (1. + refraction_ratio)).powi(2);
        r0 + (1. - r0) * (1. - cos_theta).powi(5)
    }
}

impl Scatter for Dielectric {
    fn scatter(&self, rng: &mut XorShiftRng, r: &Ray, hit: &HitRecord) -> Option<(Vec3, Ray)> {
        let refraction_ratio = if hit.front_facing {
            1. / self.refraction
        } else {
            self.refraction
        };

        let direction = r.direction().normalize();
        let cos_theta = (-direction).dot(hit.normal);
        let sin_theta = (1. - cos_theta.powi(2)).sqrt();
        let reflectance = Self::reflectance(cos_theta, refraction_ratio);

        let direction = if refraction_ratio * sin_theta > 1. || rng.gen::<f32>() < reflectance {
            reflect(direction, hit.normal)
        } else {
            refract(direction, hit.normal, refraction_ratio)
        };

        Some((Vec3::ONE, Ray::new(hit.position, direction)))
    }
}

pub type Material = Box<dyn Scatter + Send + Sync>;
