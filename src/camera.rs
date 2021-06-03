use crate::Ray;
use glam::{Vec2, Vec3};
use rand::prelude::*;
use rand_xorshift::XorShiftRng;

fn random_in_disc(rng: &mut XorShiftRng) -> Vec2 {
    loop {
        let v = Vec2::from(rng.gen::<[f32; 2]>()) * 2. - Vec2::ONE;
        let len = v.length();
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
    cu: Vec3,
    cv: Vec3,
    lens_radius: f32,
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
    ) -> Self {
        // Calculate viewport dimensions
        let theta = std::f32::consts::PI / 180.0 * vertical_fov_degrees;
        let viewport_height = 2. * (theta / 2.).tan();
        let viewport_width = aspect_ratio * viewport_height;

        // Establish a basis for the viewport
        let cw = (origin - look_at).normalize();
        let cu = up.cross(cw).normalize();
        let cv = cw.cross(cu);

        let horizontal = focus_distance * viewport_width * cu;
        let vertical = focus_distance * viewport_height * cv;

        // Projection plane's surface's low left corner point
        let lower_left_corner = origin
        - horizontal / 2. // Half viewport in x direction
        - vertical / 2. // Half viewport in y direction
        - focus_distance * cw; // Forward to viewport surface

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
            cu,
            cv,
            lens_radius: aperture / 2.,
        }
    }

    pub fn get_ray(&self, rng: &mut XorShiftRng, uv: Vec2) -> Ray {
        let rd = self.lens_radius * random_in_disc(rng);
        let offset = self.cu * rd.x + self.cv * rd.y;
        Ray::new(
            self.origin + offset,
            self.lower_left_corner + uv.x * self.horizontal + uv.y * self.vertical
                - self.origin
                - offset,
        )
    }
}
