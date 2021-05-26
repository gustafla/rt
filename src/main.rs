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
