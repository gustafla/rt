use glam::Vec3;

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

fn hit_sphere(center: Vec3, radius: f32, r: &Ray) -> f32 {
    let oc = r.origin() - center;
    let a = r.direction().dot(r.direction());
    let b = 2.0 * oc.dot(r.direction());
    let c = oc.dot(oc) - radius * radius;
    let discriminant = b * b - 4. * a * c;

    if discriminant < 0. {
        -1.
    } else {
        (-b - discriminant.sqrt()) / (2. * a)
    }
}

fn ray_color(r: &Ray) -> Color {
    let sphere = Vec3::new(0., 0., -1.);
    let t = hit_sphere(sphere, 0.5, r);
    if t > 0. {
        // Normal for the ray intersection point on the sphere surface
        let n = (r.at(t) - sphere).normalize();
        return (0.5 * Vec3::new(n.x + 1., n.y + 1., n.z + 1.)).into();
    }

    let unit_direction = r.direction().normalize();
    // From 0 to 1 when down to up
    let t = 0.5 * (unit_direction.y + 1.);
    // Blue to white gradient
    Vec3::ONE.lerp(Vec3::new(0.5, 0.7, 1.), t).into()
}

fn main() -> Result<(), image::error::ImageError> {
    // Image
    const ASPECT_RATIO: f32 = 16. / 9.;
    const IMAGE_WIDTH: u32 = 3840;
    const IMAGE_HEIGHT: u32 = (IMAGE_WIDTH as f32 / ASPECT_RATIO) as u32;

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

    // Render
    let mut buf = image::RgbImage::new(IMAGE_WIDTH, IMAGE_HEIGHT);
    for j in 0..IMAGE_HEIGHT {
        eprint!("Scanlines remaining: {:>5}\r", IMAGE_HEIGHT - j - 1);
        for i in 0..IMAGE_WIDTH {
            // Ray through viewport in right handed space
            let u = i as f32 / (IMAGE_WIDTH - 1) as f32; // i to u
            let v = 1. - (j as f32 / (IMAGE_HEIGHT - 1) as f32); // j (y down) to v (y up)
            let uv_on_plane = lower_left_corner + u * horizontal + v * vertical;
            let r = Ray::new(origin, uv_on_plane - origin);

            // Store color seen in this pixel
            buf.put_pixel(i, j, ray_color(&r).into());
        }
    }

    // Output file
    let output_file_path = std::env::args_os()
        .nth(1)
        .unwrap_or_else(|| std::ffi::OsString::from("image.png"));
    buf.save(&output_file_path)?;
    eprintln!("\nDone.");
    std::process::Command::new("imv")
        .args(&[output_file_path])
        .output()
        .ok();
    Ok(())
}
