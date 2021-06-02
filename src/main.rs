use glam::{Vec2, Vec3};
use parking_lot::Mutex;
use rand::prelude::*;
use rand_xorshift::XorShiftRng;
use std::convert::TryFrom;
use std::error::Error;
use std::ops::Range;
use std::path::Path;

struct Color(Vec3);

impl From<Vec3> for Color {
    fn from(v: Vec3) -> Self {
        Self(v)
    }
}

impl From<Color> for Vec3 {
    fn from(c: Color) -> Self {
        c.0
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

const COLOR_CHANNELS: usize = 3;
type OutputColor = [u8; COLOR_CHANNELS];

impl From<Color> for OutputColor {
    fn from(color: Color) -> Self {
        let c = Vec3::from(color.sqrt().clamp(0., 0.999)) * 256.;
        [c.x as u8, c.y as u8, c.z as u8]
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
    pub fn new(vertical_fov_degrees: f32, aspect_ratio: f32) -> Self {
        let focal_length = 1.;
        let theta = std::f32::consts::PI / 180.0 * vertical_fov_degrees;
        let viewport_height = 2. * (theta / 2.).tan();
        let viewport_width = aspect_ratio * viewport_height;

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

trait Scatter {
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

struct Lambertian {
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

struct Metal {
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
        let reflected = reflect(r.direction, hit.normal).normalize();
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

struct Dielectric {
    refraction: f32,
}

impl Dielectric {
    pub fn new(refraction: f32) -> Self {
        Self { refraction }
    }

    fn reflectance(&self, cos_theta: f32) -> f32 {
        // Schlick's approximation
        let r0 = ((1. - self.refraction) / (1. + self.refraction)).powi(2);
        r0 * (1. - r0) * (1. - cos_theta).powi(5)
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
        let reflectance = self.reflectance(cos_theta);

        let direction = if refraction_ratio * sin_theta > 1. || rng.gen::<f32>() < reflectance {
            reflect(direction, hit.normal)
        } else {
            refract(direction, hit.normal, refraction_ratio)
        };

        Some((Vec3::ONE, Ray::new(hit.position, direction)))
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

type Surface = Box<dyn Hit + Send + Sync>;
type Material = Box<dyn Scatter + Send + Sync>;

struct Object {
    pub surface: Surface,
    pub material: Material,
}

struct World {
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

fn ray_color(r: Ray, world: &World, rng: &mut XorShiftRng, depth: u32) -> Vec3 {
    if depth == 0 {
        return Vec3::ZERO;
    }

    if let Some((hit, material)) = world.traverse(&r, 0.001) {
        if let Some((att, r)) = material.scatter(rng, &r, &hit) {
            att * ray_color(r, world, rng, depth - 1)
        } else {
            Vec3::ZERO
        }
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
    const IMAGE_WIDTH: usize = 3840;
    const IMAGE_HEIGHT: usize = (IMAGE_WIDTH as f32 / ASPECT_RATIO) as usize;
    const SAMPLES_PER_PIXEL: u32 = 64;
    const MAX_DEPTH: u32 = 8;

    // World
    let world = World::new(vec![
        // Ground
        Object {
            surface: Box::new(Sphere::new(Vec3::new(0., -100.5, -1.), 100.)),
            material: Box::new(Lambertian::new(Vec3::new(0.8, 0.8, 0.0))),
        },
        // Center
        Object {
            surface: Box::new(Sphere::new(Vec3::new(0., 0., -1.), 0.5)),
            material: Box::new(Lambertian::new(Vec3::new(0.1, 0.2, 0.5))),
        },
        // Left
        Object {
            surface: Box::new(Sphere::new(Vec3::new(-1., 0., -1.), 0.5)),
            material: Box::new(Dielectric::new(1.5)),
        },
        // Right
        Object {
            surface: Box::new(Sphere::new(Vec3::new(1., 0., -1.), 0.5)),
            material: Box::new(Metal::new(Vec3::new(0.8, 0.6, 0.2), 1.)),
        },
    ]);

    // Camera
    let camera = Camera::new(90., ASPECT_RATIO);

    // Render using all cpu cores
    let nthreads = num_cpus::get();
    // Allocate image buffer
    let mut pixel_data = vec![0u8; IMAGE_WIDTH * IMAGE_HEIGHT * COLOR_CHANNELS];
    // Divide buffer into chunks for threads to work on
    const CHUNK_PIXELS: usize = 4096;
    let chunks: Mutex<Vec<_>> = Mutex::new(
        pixel_data
            .chunks_mut(CHUNK_PIXELS * COLOR_CHANNELS)
            .enumerate()
            .collect(),
    );
    // Run the rendering threads
    crossbeam_utils::thread::scope(|s| {
        for _ in 0..nthreads {
            s.spawn(|_| {
                let mut rng = XorShiftRng::seed_from_u64(123);

                while let Some((i, chunk)) = {
                    let tmp = chunks.lock().pop();
                    tmp // Drop mutex guard
                } {
                    let chunk_offset = CHUNK_PIXELS * i;
                    for i in 0..chunk.len() / COLOR_CHANNELS {
                        // Calculate pixel coordinates
                        let pixel = chunk_offset + i;
                        let xy = Vec2::new(
                            (pixel % IMAGE_WIDTH) as f32,
                            (IMAGE_HEIGHT - 1 - (pixel / IMAGE_WIDTH)) as f32,
                        );

                        // Accumulate color from rays
                        let mut color = Vec3::ZERO;
                        for _ in 0..SAMPLES_PER_PIXEL {
                            // Ray through viewport in right handed space
                            let random = Vec2::from(rng.gen::<[f32; 2]>());
                            let wh = Vec2::new(IMAGE_WIDTH as f32, IMAGE_HEIGHT as f32);
                            let uv = (xy + random) / (wh - Vec2::ONE);
                            color += ray_color(camera.get_ray(uv), &world, &mut rng, MAX_DEPTH);
                        }

                        // Average samples, clamp and output to 8bpp RGB buffer
                        chunk[i * COLOR_CHANNELS..][..COLOR_CHANNELS].copy_from_slice(
                            &OutputColor::from(Color::from(color / SAMPLES_PER_PIXEL as f32)),
                        );
                    }
                }
            });
        }
    })
    .unwrap();

    // Encode PNG from results
    let output_file_path = std::env::args_os()
        .nth(1)
        .unwrap_or_else(|| std::ffi::OsString::from("image.png"));
    write_png(&output_file_path, IMAGE_WIDTH, IMAGE_HEIGHT, &pixel_data).unwrap();
    eprintln!("\nDone.");
    std::process::Command::new("imv")
        .args(&[output_file_path])
        .spawn()
        .ok();
}

fn write_png(
    path: impl AsRef<Path>,
    width: usize,
    height: usize,
    rgb8_data: &[u8],
) -> Result<(), Box<dyn Error>> {
    let file = std::fs::File::create(&path)?;
    let w = std::io::BufWriter::new(file);
    let mut encoder = png::Encoder::new(w, u32::try_from(width)?, u32::try_from(height)?);
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&rgb8_data)?;
    Ok(())
}
