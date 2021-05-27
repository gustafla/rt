use glam::Vec3;

trait Color {
    fn format_color(self) -> String;
}

impl Color for Vec3 {
    fn format_color(self) -> String {
        format!(
            "{} {} {}",
            (255.999 * self.x) as u8,
            (255.999 * self.y) as u8,
            (255.999 * self.z) as u8,
        )
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

fn ray_color(r: &Ray) -> impl Color {
    let unit_direction = r.direction().normalize();
    // From 0 to 1 when down to up
    let t = 0.5 * (unit_direction.y + 1.);
    // Blue to white gradient
    Vec3::new(0.5, 0.7, 1.).lerp(Vec3::ONE, t)
}

fn main() {
    // Image
    const ASPECT_RATIO: f32 = 16. / 9.;
    const IMAGE_WIDTH: u32 = 256;
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
    println!("P3");
    println!("{} {}", IMAGE_WIDTH, IMAGE_HEIGHT);
    println!("255");
    for j in 0..IMAGE_HEIGHT {
        eprint!("Scanlines remaining: {:>5}\r", IMAGE_HEIGHT - j - 1);
        for i in 0..IMAGE_WIDTH {
            let u = i as f32 / (IMAGE_WIDTH - 1) as f32;
            let v = j as f32 / (IMAGE_HEIGHT - 1) as f32;
            let uv_on_plane = lower_left_corner + u * horizontal + v * vertical;

            let r = Ray::new(origin, uv_on_plane - origin);

            let pixel_color = ray_color(&r);
            println!("{}", pixel_color.format_color());
        }
    }
    eprintln!("\nDone.");
}
