use colorgrad::Gradient;
use image::{ImageBuffer, Rgb, RgbImage};
use rayon::prelude::*;

use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::cli_setup::Fractal;
use crate::cli_setup::UserVars;

mod cli_setup;

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Copy)]
pub struct IVec2 {
    pub x: u64,
    pub y: u64,
}
fn main() {
    let user_vars = UserVars::new();

    let timer = Instant::now();
    let image = render(
        user_vars.image_size,
        user_vars.constant,
        user_vars.zoom,
        user_vars.zoom_coords,
        user_vars.iterations,
        user_vars.gradient,
        user_vars.fractal_type,
    );
    let duration = timer.elapsed().as_millis();

    image.save(user_vars.out_name).unwrap();

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

fn compute_next_juliacubed(current: Vec2, constant: Vec2) -> Vec2 {
    let z_real = (current.x * current.x * current.x) - (3.0 * current.x * current.y * current.y);
    let z_imaginary = (3.0 * current.x * current.x * current.y) - (current.y * current.y * current.y);

    Vec2 {
        x: z_real + constant.x,
        y: z_imaginary + constant.y,
    }
}

fn compute_next_mandelbrot(current: Vec2, constant: Vec2, zoom: f64) -> Vec2 {
    let z_real = (current.x * current.x) - (current.y * current.y);
    let z_imaginary = 2.0 * current.x * current.y;

    Vec2 {
        x: z_real + constant.x / zoom,
        y: z_imaginary + constant.y / zoom,
    }
}

fn modulus_squared(z: Vec2) -> f64 {
    (z.x * z.x) + (z.y * z.y)
}

// Zn = Zn-1 + C
fn iterate_to_max(
    initial_z: Vec2,
    constant: Vec2,
    fractal_zoom: f64,
    fractal_type: Fractal,
    max_iterations: usize,
) -> usize {
    let mut zn = match fractal_type {
        Fractal::Julia | Fractal::JuliaCubed => Vec2 {
            x: initial_z.x / fractal_zoom,
            y: initial_z.y / fractal_zoom,
        },
        Fractal::Mandelbrot => Vec2 {
            x: initial_z.x,
            y: initial_z.y,
        },
    };

    let mut iteration = 0;
    while modulus_squared(zn) < 4.0 && iteration < max_iterations {
        zn = match fractal_type {
            Fractal::Julia => compute_next_julia(zn, constant),
            Fractal::JuliaCubed => compute_next_juliacubed(zn, constant),
            Fractal::Mandelbrot => compute_next_mandelbrot(zn, constant, fractal_zoom),
        };
        iteration += 1;
    }

    iteration
}

fn render(
    render_size: IVec2,
    constant: Vec2,
    fractal_zoom: f64,
    zoom_coords: Vec2,
    max_iterations: usize,
    gradient: Gradient,
    fractal_type: Fractal,
) -> RgbImage {
    let buffer: RgbImage = ImageBuffer::new(
        render_size.x.try_into().unwrap(),
        render_size.y.try_into().unwrap(),
    );

    let scale = 1.0 / (render_size.y as f64 / 2.0);
    let image = Arc::new(Mutex::new(buffer));
    (0..render_size.y).into_par_iter().for_each(|y| {
        (0..render_size.x).into_par_iter().for_each(|x| {
            let pixel_x =
                ((x as f64 - render_size.x as f64 / 2.0) * scale) + zoom_coords.x * fractal_zoom;
            let pixel_y =
                ((y as f64 - render_size.y as f64 / 2.0) * scale) + zoom_coords.y * fractal_zoom;
            // println!("for ({}, {}), it gave: {}, {}", x, y, pixel_x, pixel_y);

            let iterations = match fractal_type {
                Fractal::Julia => iterate_to_max(
                    Vec2 {
                        x: pixel_x,
                        y: pixel_y,
                    },
                    constant,
                    fractal_zoom,
                    Fractal::Julia,
                    max_iterations,
                ),
                Fractal::JuliaCubed => iterate_to_max(
                    Vec2 {
                        x: pixel_x,
                        y: pixel_y,
                    },
                    constant,
                    fractal_zoom,
                    Fractal::JuliaCubed,
                    max_iterations,
                ),
                Fractal::Mandelbrot => iterate_to_max(
                    Vec2 { x: 0.0, y: 0.0 },
                    Vec2 {
                        x: pixel_x,
                        y: pixel_y,
                    },
                    fractal_zoom,
                    Fractal::Mandelbrot,
                    max_iterations,
                ),
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

    let image = &*image.lock().unwrap();

    image.to_owned()
}
