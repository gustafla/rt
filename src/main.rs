use glam::Vec3;
use std::convert::TryFrom;
use std::ops::Range;

struct Color(Vec3);

impl From<Color> for image::Rgb<u8> {
    fn from(color: Color) -> Self {
        Self([
            (255.999 * color.0.x) as u8,
            (255.999 * color.0.y) as u8,
            (255.999 * color.0.z) as u8,
        ])
    }
}

impl From<Vec3> for Color {
    fn from(v: Vec3) -> Self {
        Self(v)
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

fn ray_color(r: &Ray, world: &World) -> Color {
    if let Some(hit) = world.hit(r, 0.0..f32::INFINITY) {
        (0.5 * (hit.normal + Vec3::ONE)).into()
    } else {
        let unit_direction = r.direction().normalize();
        // From 0 to 1 when down to up
        let t = 0.5 * (unit_direction.y + 1.);
        // Blue to white gradient
        Vec3::ONE.lerp(Vec3::new(0.5, 0.7, 1.), t).into()
    }
}

fn main() {
    // Image
    const ASPECT_RATIO: f32 = 16. / 9.;
    const IMAGE_WIDTH: u32 = 7680;
    const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f32 / ASPECT_RATIO) as u32;

    // World
    let world = vec![
        Box::new(Sphere::new(Vec3::new(0., 0., -1.), 0.5)) as WorldItem,
        Box::new(Sphere::new(Vec3::new(0., -100.5, -1.), 100.)),
    ];

    // Camera
    let viewport_height = 2.;
    let viewport_width = ASPECT_RATIO * viewport_height;
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

    // Render using all cpu cores
    let nthreads = u32::try_from(num_cpus::get()).unwrap().min(IMAGE_HEIGHT);
    let lines_per_thread = IMAGE_HEIGHT / nthreads;
    let rounding_error_lines = IMAGE_HEIGHT - lines_per_thread * nthreads;
    let subpixel_data = crossbeam::scope(|s| {
        let threads: Vec<_> = (0..nthreads)
            .into_iter()
            .map(|thread| {
                let world = &world;
                s.spawn(move |_| {
                    // Account for rounding error in the first thread
                    let lines = if thread == 0 {
                        lines_per_thread + rounding_error_lines
                    } else {
                        lines_per_thread
                    };

                    // Color the pixels
                    let mut buf = image::RgbImage::new(IMAGE_WIDTH, lines);
                    for (j, line) in buf.enumerate_rows_mut() {
                        let mut j = thread * lines_per_thread + j;
                        if thread == 0 {
                            eprint!("Scanlines remaining: {:>5}\r", lines - j - 1);
                        } else {
                            j += rounding_error_lines;
                        }

                        for (i, _, pixel) in line {
                            // Ray through viewport in right handed space
                            let u = i as f32 / (IMAGE_WIDTH - 1) as f32;
                            let v = 1. - (j as f32 / (IMAGE_HEIGHT - 1) as f32);
                            let uv_on_plane = lower_left_corner + u * horizontal + v * vertical;
                            let r = Ray::new(origin, uv_on_plane - origin);

                            // Store color seen in this pixel
                            *pixel = ray_color(&r, world).into();
                        }
                    }

                    // Return the buffer to be concatenated
                    buf.into_vec()
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

    // Construct image from results
    let output = image::RgbImage::from_vec(IMAGE_WIDTH, IMAGE_HEIGHT, subpixel_data).unwrap();

    // Output file
    let output_file_path = std::env::args_os()
        .nth(1)
        .unwrap_or_else(|| std::ffi::OsString::from("image.png"));
    output.save(&output_file_path).unwrap();
    eprintln!("\nDone.");
    std::process::Command::new("imv")
        .args(&[output_file_path])
        .output()
        .ok();
}
