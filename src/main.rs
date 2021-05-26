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

fn main() {
    const IMAGE_WIDTH: u64 = 256;
    const IMAGE_HEIGHT: u64 = 256;

    println!("P3");
    println!("{} {}", IMAGE_WIDTH, IMAGE_HEIGHT);
    println!("255");
    for j in 0..IMAGE_HEIGHT {
        eprint!("Scanlines remaining: {:>5}\r", IMAGE_HEIGHT - j - 1);
        for i in 0..IMAGE_WIDTH {
            let pixel_color = Vec3::new(
                (i as f32) / (IMAGE_WIDTH - 1) as f32,
                (j as f32) / (IMAGE_HEIGHT - 1) as f32,
                0.25,
            );

            println!("{}", pixel_color.format_color());
        }
    }
    eprintln!("\nDone.");
}
