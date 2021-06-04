use crate::Ray;
use rand::prelude::*;
use std::ops::Range;
use ultraviolet::{Vec2, Vec3};

fn random_in_disc(rng: &mut impl Rng) -> Vec2 {
    loop {
        let v = Vec2::from(rng.gen::<[f32; 2]>()) * 2. - Vec2::one();
        let len = v.mag_sq();
        if len < 1. {
            return v;
        }
    }
}

pub struct Camera {
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
    u: Vec3,
    v: Vec3,
    lens_radius: f32,
    shutter_time: Range<f32>,
}

impl Camera {
    pub fn new(
        origin: Vec3,
        look_at: Vec3,
        up: Vec3,
        vertical_fov_degrees: f32,
        aspect_ratio: f32,
        aperture: f32,
        focus_distance: f32,
        shutter_time: Range<f32>,
    ) -> Self {
        // Calculate viewport dimensions
        let theta = std::f32::consts::PI / 180.0 * vertical_fov_degrees;
        let viewport_height = 2. * (theta / 2.).tan();
        let viewport_width = aspect_ratio * viewport_height;

        // Establish a basis for the viewport
        let w = (origin - look_at).normalized();
        let u = up.cross(w).normalized();
        let v = w.cross(u);

        let horizontal = focus_distance * viewport_width * u;
        let vertical = focus_distance * viewport_height * v;

        // Projection plane's surface's low left corner point
        let lower_left_corner = origin
        - horizontal / 2. // Half viewport in x direction
        - vertical / 2. // Half viewport in y direction
        - focus_distance * w; // Forward to viewport surface

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            u,
            v,
            lens_radius: aperture / 2.,
            shutter_time,
        }
    }

    pub fn get_ray(&self, rng: &mut impl Rng, uv: Vec2) -> Ray {
        let rd = self.lens_radius * random_in_disc(rng);
        let offset = self.u * rd.x + self.v * rd.y;
        Ray::new(
            self.origin + offset,
            self.lower_left_corner + uv.x * self.horizontal + uv.y * self.vertical
                - self.origin
                - offset,
            rng.gen_range(self.shutter_time.clone()),
        )
    }
}
