use colorgrad::{Color, Gradient};
use image::{ImageBuffer, Rgb, RgbImage};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::{time::Instant};

#[derive(Debug, Clone, Copy)]
struct Vec2 {
    x: f64,
    y: f64,
}

#[derive(Debug, Clone, Copy)]
struct IVec2 {
    x: u64,
    y: u64,
}

enum Fractal {
    Julia,
    Mandelbrot,
}

fn main() {
    let image_size = IVec2 { x: 2560, y: 1440 };
    let grayscale = colorgrad::CustomGradient::new()
        .colors(&[
            Color::from_rgba8(0, 0, 0, 255),
            Color::from_rgba8(255, 255, 255, 255),
        ])
        .build()
        .unwrap();
    let output_buffer: RgbImage = ImageBuffer::new(
        image_size.x.try_into().unwrap(),
        image_size.y.try_into().unwrap(),
    );

    // let constant = Vec2 { x: -0.4, y: 0.6};
    let constant = Vec2 { x: -0.8, y: 0.156 };

    let timer = Instant::now();
    render(
        image_size,
        constant,
        1.0,
        500,
        output_buffer,
        grayscale,
        Fractal::Julia,
        "julia.png",
    );
    let duration = timer.elapsed().as_millis();
    println!("calculation duration: {} ms", duration);
}

fn compute_next_julia(current: Vec2, constant: Vec2) -> Vec2 {
    let z_real = (current.x * current.x) - (current.y * current.y);
    let z_imaginary = 2.0 * current.x * current.y;

    Vec2 {
        x: z_real + constant.x,
        y: z_imaginary + constant.y,
    }
}

fn modulus_squared(z: Vec2) -> f64 {
    (z.x * z.x) + (z.y * z.y)
}

// Zn = Zn-1 + C
fn iterate_to_max_julia(initial_z: Vec2, constant: Vec2, fractal_zoom: f64, max_iterations: usize) -> usize {
    let mut zn = Vec2 { x: initial_z.x / fractal_zoom, y: initial_z.y / fractal_zoom };
    let mut iteration = 0;
    while modulus_squared(zn) < 4.0 && iteration < max_iterations {
        zn = compute_next_julia(zn, constant);
        iteration += 1;
    }

    iteration
}

fn render(
    render_size: IVec2,
    constant: Vec2,
    fractal_zoom: f64,
    max_iterations: usize,
    buffer: RgbImage,
    gradient: Gradient,
    fractal_type: Fractal,
    out_name: &str,
) {
    let scale = 1.0 / (render_size.y as f64 / 2.0);
    let image = Arc::new(Mutex::new(buffer));
    (0..render_size.y).into_par_iter().for_each(|y| {
        (0..render_size.x).into_par_iter().for_each(|x| {
            let pixel_x = (x as f64 - render_size.x as f64 / 2.0) * scale;
            let pixel_y = (y as f64 - render_size.y as f64 / 2.0) * scale;

            let iterations = match fractal_type {
                Fractal::Julia => iterate_to_max_julia(
                    Vec2 {
                        x: pixel_x,
                        y: pixel_y,
                    },
                    constant,
                    fractal_zoom,
                    max_iterations,
                ),
                _ => todo!(),
            };

            let normalised_value = iterations as f64 / max_iterations as f64;
            let colour = gradient.at(normalised_value).to_rgba8();

            let mut image = image.lock().unwrap();
            image.put_pixel(
                x.try_into().unwrap(),
                y.try_into().unwrap(),
                Rgb([colour[0], colour[1], colour[2]]),
            );
        });
    });

    let image = image.lock().unwrap();
    image.save(out_name).unwrap();
}
