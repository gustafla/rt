mod camera;
mod color;
mod ray;
mod world;

use camera::Camera;
use color::{Color, OutputColor, COLOR_CHANNELS};
use glam::{Vec2, Vec3};
use parking_lot::Mutex;
use rand::prelude::*;
use rand_xorshift::XorShiftRng;
use ray::Ray;
use std::convert::TryFrom;
use std::error::Error;
use std::path::Path;
use world::{
    material::{Dielectric, Lambertian, Metal},
    surface::Sphere,
    Object, World,
};

fn ray_color(r: Ray, world: &World, rng: &mut XorShiftRng, depth: u32) -> Vec3 {
    if depth == 0 {
        return Vec3::ZERO;
    }

    if let Some((hit, material)) = world.traverse(&r, 0.0001) {
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
    const MAX_DEPTH: u32 = 32;

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
        Object {
            surface: Box::new(Sphere::new(Vec3::new(-1., 0., -1.), -0.45)),
            material: Box::new(Dielectric::new(1.5)),
        },
        // Right
        Object {
            surface: Box::new(Sphere::new(Vec3::new(1., 0., -1.), 0.5)),
            material: Box::new(Metal::new(Vec3::new(0.8, 0.6, 0.2), 0.)),
        },
    ]);

    // Camera
    let lookfrom = Vec3::new(3., 3., 2.);
    let lookat = -Vec3::Z;
    let camera = Camera::new(
        lookfrom,
        lookat,
        Vec3::Y,
        20.,
        ASPECT_RATIO,
        0.1,
        lookfrom.distance(lookat),
    );

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
                            color += ray_color(
                                camera.get_ray(&mut rng, uv),
                                &world,
                                &mut rng,
                                MAX_DEPTH,
                            );
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
