use glam::{Vec2, Vec3};
use rand::prelude::*;
use rand_xorshift::XorShiftRng;
use std::convert::TryFrom;
use std::ops::Range;

const COLOR_CHANNELS: usize = 3;
type OutputColor = [u8; COLOR_CHANNELS];
struct Color(Vec3);

impl From<Color> for OutputColor {
    fn from(color: Color) -> Self {
        let c = color.0 * 256.;
        [c.x as u8, c.y as u8, c.z as u8]
    }
}

impl From<Vec3> for Color {
    fn from(v: Vec3) -> Self {
        Self(v)
    }
}

impl Color {
    fn sqrt(self) -> Self {
        Self(Vec3::new(self.0.x.sqrt(), self.0.y.sqrt(), self.0.z.sqrt()))
    }

    fn clamp(self, min: f32, max: f32) -> Color {
        Self(self.0.clamp(Vec3::splat(min), Vec3::splat(max)))
    }
}

fn random_vec3(rng: &mut impl Rng, r: Range<f32>) -> Vec3 {
    Vec3::new(
        rng.gen_range(r.clone()),
        rng.gen_range(r.clone()),
        rng.gen_range(r.clone()),
    )
}

fn random_vec3_in_sphere(rng: &mut impl Rng) -> Vec3 {
    loop {
        let v = random_vec3(rng, -1.0..1.);
        if v.length() < 1. {
            return v;
        }
    }
}

struct Ray {
    origin: Vec3,
    direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    pub fn origin(&self) -> Vec3 {
        self.origin
    }

    pub fn direction(&self) -> Vec3 {
        self.direction
    }

    pub fn at(&self, t: f32) -> Vec3 {
        self.origin + t * self.direction
    }
}

struct Camera {
    origin: Vec3,
    lower_left_corner: Vec3,
    horizontal: Vec3,
    vertical: Vec3,
}

impl Camera {
    pub fn new(aspect_ratio: f32) -> Self {
        let viewport_height = 2.;
        let viewport_width = aspect_ratio * viewport_height;
        let focal_length = 1.;

        let origin = Vec3::ZERO;
        let horizontal = Vec3::new(viewport_width, 0., 0.);
        let vertical = Vec3::new(0., viewport_height, 0.);
        // Projection plane's surface's low left corner point
        let lower_left_corner = origin
        - horizontal / 2. // Half viewport in x direction
        - vertical / 2. // Half viewport in y direction
        - Vec3::new(
            0.,
            0.,
            focal_length, /* To viewport in z direction. Because of the right handed coordinates */
                          /* this actually makes the vector point forward (towards negative z) */
        );

        Self {
            origin,
            lower_left_corner,
            horizontal,
            vertical,
        }
    }

    pub fn get_ray(&self, uv: Vec2) -> Ray {
        Ray::new(
            self.origin,
            self.lower_left_corner + uv.x * self.horizontal + uv.y * self.vertical - self.origin,
        )
    }
}

struct HitRecord {
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

trait Hit {
    fn hit(&self, r: &Ray, t_range: Range<f32>) -> Option<HitRecord>;
}

type WorldItem = Box<dyn Hit + Send + Sync>;
type World = [WorldItem];

impl Hit for World {
    fn hit(&self, r: &Ray, t_range: Range<f32>) -> Option<HitRecord> {
        let mut nearest_hit = None;
        let mut nearest_t = t_range.end;

        for surface in self.iter() {
            if let Some(hit) = surface.hit(r, t_range.start..nearest_t) {
                nearest_t = hit.t;
                nearest_hit = Some(hit);
            }
        }

        nearest_hit
    }
}

struct Sphere {
    center: Vec3,
    radius: f32,
}

impl Sphere {
    fn new(center: Vec3, radius: f32) -> Self {
        Self { center, radius }
    }
}

impl Hit for Sphere {
    fn hit(&self, r: &Ray, t_range: Range<f32>) -> Option<HitRecord> {
        let oc = r.origin() - self.center;
        let a = r.direction().length().powi(2);
        let half_b = oc.dot(r.direction());
        let c = oc.length().powi(2) - self.radius.powi(2);

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
        let outward_normal = (position - self.center) / self.radius;
        Some(HitRecord::new(position, outward_normal, root, r))
    }
}

fn ray_color(r: Ray, world: &World, rng: &mut impl Rng, depth: u32) -> Vec3 {
    if depth <= 0 {
        return Vec3::ZERO;
    }

    if let Some(hit) = world.hit(&r, 0.001..f32::INFINITY) {
        // Diffuse ray (lambertian-ish distribution)
        let target = hit.position + hit.normal + random_vec3_in_sphere(rng).normalize();
        let r = Ray::new(hit.position, target - hit.position);
        0.5 * ray_color(r, world, rng, depth - 1)
    } else {
        let unit_direction = r.direction().normalize();
        // From 0 to 1 when down to up
        let t = 0.5 * (unit_direction.y + 1.);
        // Blue to white gradient
        Vec3::ONE.lerp(Vec3::new(0.5, 0.7, 1.), t)
    }
}

fn main() {
    // Image
    const ASPECT_RATIO: f32 = 16. / 9.;
    const IMAGE_WIDTH: u32 = 3840;
    const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f32 / ASPECT_RATIO) as u32;
    const SAMPLES_PER_PIXEL: u32 = 256;
    const MAX_DEPTH: u32 = 8;

    // World
    let world = vec![
        Box::new(Sphere::new(Vec3::new(0., 0., -1.), 0.5)) as WorldItem,
        Box::new(Sphere::new(Vec3::new(0., -100.5, -1.), 100.)),
    ];

    // Camera
    let camera = Camera::new(ASPECT_RATIO);

    // Render using all cpu cores
    let nthreads = u32::try_from(num_cpus::get()).unwrap().min(IMAGE_HEIGHT);
    let lines_per_thread = IMAGE_HEIGHT / nthreads;
    let rounding_error_lines = IMAGE_HEIGHT - lines_per_thread * nthreads;
    let pixel_data: Vec<u8> = crossbeam::scope(|s| {
        let threads: Vec<_> = (0..nthreads)
            .into_iter()
            .map(|thread| {
                let world = &world;
                let camera = &camera;
                s.spawn(move |_| {
                    // Account for rounding error in the first thread
                    let lines = if thread == 0 {
                        lines_per_thread + rounding_error_lines
                    } else {
                        lines_per_thread
                    };

                    // Allocate storage for output pixel data
                    let mut buf = Vec::with_capacity(
                        usize::try_from(IMAGE_WIDTH).unwrap()
                            * usize::try_from(lines).unwrap()
                            * COLOR_CHANNELS,
                    );

                    // Render
                    let mut rng = XorShiftRng::seed_from_u64(thread.into());
                    for j in 0..lines {
                        let mut j = thread * lines_per_thread + j;
                        if thread == 0 {
                            eprint!("Scanlines remaining: {:>5}\r", lines - j - 1);
                        } else {
                            j += rounding_error_lines;
                        }

                        for i in 0..IMAGE_WIDTH {
                            let mut color = Vec3::ZERO;
                            for _ in 0..SAMPLES_PER_PIXEL {
                                // Ray through viewport in right handed space
                                let random = Vec2::new(rng.gen(), rng.gen());
                                let ij = Vec2::new(i as f32, (IMAGE_HEIGHT - 1 - j) as f32);
                                let wh = Vec2::new(IMAGE_WIDTH as f32, IMAGE_HEIGHT as f32);
                                let uv = (ij + random) / (wh - Vec2::ONE);

                                // Accumulate color seen in this sample
                                color += ray_color(camera.get_ray(uv), world, &mut rng, MAX_DEPTH);
                            }

                            // Average samples, clamp and output to 8bpp RGB buffer
                            buf.extend_from_slice(&OutputColor::from(
                                Color(color / SAMPLES_PER_PIXEL as f32)
                                    .sqrt()
                                    .clamp(0., 0.9999),
                            ));
                        }
                    }

                    // Return the buffer to be concatenated
                    buf
                })
            })
            .collect();

        // Concatenate thread results to a vec of subpixels
        threads
            .into_iter()
            .flat_map(|t| t.join().unwrap())
            .collect()
    })
    .unwrap();

    // Encode PNG from results
    let output_file_path = std::env::args_os()
        .nth(1)
        .unwrap_or_else(|| std::ffi::OsString::from("image.png"));
    {
        let file = std::fs::File::create(&output_file_path).unwrap();
        let w = std::io::BufWriter::new(file);
        let mut encoder = png::Encoder::new(w, IMAGE_WIDTH, IMAGE_HEIGHT);
        encoder.set_color(png::ColorType::RGB);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&pixel_data).unwrap();
    }
    eprintln!("\nDone.");
    std::process::Command::new("imv")
        .args(&[output_file_path])
        .spawn()
        .ok();
}
