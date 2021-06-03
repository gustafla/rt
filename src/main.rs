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
use std::{
    convert::TryFrom,
    error::Error,
    ffi::OsString,
    fs::File,
    io::{prelude::*, BufWriter},
    time::SystemTime,
};
use world::World;

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
    let mut args = pico_args::Arguments::from_env();

    // Image
    const MAX_DEPTH: u32 = 64;
    let aspect_ratio: f32 = args
        .opt_value_from_fn(["-a", "--aspect-ratio"], |s| {
            let mut split = s.splitn(2, ':');
            match (split.next(), split.next()) {
                (Some(s1), Some(s2)) => Ok(s1.parse::<f32>()? / s2.parse::<f32>()?),
                (Some(s), None) => s.parse(),
                _ => unreachable!(),
            }
        })
        .unwrap()
        .unwrap_or(16. / 9.);
    let image_height: usize = args
        .opt_value_from_fn(["-h", "--height"], |s| match s {
            "HD" | "720p" => Ok(720),
            "FHD" | "1080p" => Ok(1080),
            "4K" | "2160p" => Ok(2160),
            "8K" | "4320p" => Ok(4320),
            _ => s.parse(),
        })
        .unwrap()
        .unwrap_or(720);
    let image_width: usize = (image_height as f32 * aspect_ratio) as usize;
    let samples_per_pixel: u32 = args
        .opt_value_from_str(["-s", "--samples"])
        .unwrap()
        .unwrap_or(64);
    let mut remaining = args.finish();
    let output_file_path = remaining.pop().unwrap_or_else(|| {
        OsString::from(format!(
            "{}.png",
            humantime::format_rfc3339(SystemTime::now())
        ))
    });
    (!remaining.is_empty()).then(|| panic!("Unknown arguments: {:?}", remaining));
    // Ensure output file is writable before starting a long render
    let output_file_writer = BufWriter::new(File::create(output_file_path).unwrap());

    // World
    let world = World::random(&mut XorShiftRng::seed_from_u64(42));

    // Camera
    let lookfrom = Vec3::new(13., 2., 3.);
    let lookat = Vec3::ZERO;
    let camera = Camera::new(lookfrom, lookat, Vec3::Y, 20., aspect_ratio, 0.1, 10.);

    // Render using all cpu cores
    let nthreads = num_cpus::get();
    // Allocate image buffer
    let mut pixel_data = vec![0u8; image_width * image_height * COLOR_CHANNELS];
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
                    let (chunk, len) = {
                        let mut chunks = chunks.lock();
                        (chunks.pop(), chunks.len())
                    };
                    eprint!("Chunks left {:>5}\r", len);
                    chunk
                } {
                    let chunk_offset = CHUNK_PIXELS * i;
                    for i in 0..chunk.len() / COLOR_CHANNELS {
                        // Calculate pixel coordinates
                        let pixel = chunk_offset + i;
                        let xy = Vec2::new(
                            (pixel % image_width) as f32,
                            (image_height - 1 - (pixel / image_width)) as f32,
                        );

                        // Accumulate color from rays
                        let mut color = Vec3::ZERO;
                        for _ in 0..samples_per_pixel {
                            // Ray through viewport in right handed space
                            let random = Vec2::from(rng.gen::<[f32; 2]>());
                            let wh = Vec2::new(image_width as f32, image_height as f32);
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
                            &OutputColor::from(Color::from(color / samples_per_pixel as f32)),
                        );
                    }
                }
            });
        }
    })
    .unwrap();

    // Encode PNG from results
    write_png(output_file_writer, image_width, image_height, &pixel_data).unwrap();
    eprintln!("Done.                  ");
}

fn write_png(
    write: impl Write,
    width: usize,
    height: usize,
    rgb8_data: &[u8],
) -> Result<(), Box<dyn Error>> {
    let mut encoder = png::Encoder::new(write, u32::try_from(width)?, u32::try_from(height)?);
    encoder.set_color(png::ColorType::RGB);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    writer.write_image_data(&rgb8_data)?;
    Ok(())
}
