use clap::{Arg, Command};
use colorgrad::{Color, Gradient};
use image::{ImageBuffer, Rgb, RgbImage};
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Instant;

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
    // CLI setup
    let matches = Command::new("fractl")
        .version("0.1.0")
        .author("Jackson Ly (JumpyJacko)")
        .about("A small fractal renderer")
        .arg(
            Arg::new("fractal")
                .short('f')
                .long("fractal")
                .default_value("julia")
                .help("Fractal type (Julia, Mandelbrot)"),
        )
        .arg(
            Arg::new("iterations")
                .short('i')
                .long("iterations")
                .default_value("500")
                .help("Amount of iterations"),
        )
        .arg(
            Arg::new("width")
                .short('w')
                .long("width")
                .default_value("1920")
                .help("Width of output image"),
        )
        .arg(
            Arg::new("height")
                .short('v')
                .long("height")
                .default_value("1080")
                .help("Height of output image"),
        )
        .arg(
            Arg::new("name")
                .short('o')
                .long("output-name")
                .default_value("output.png")
                .help("Name of output image"),
        )
        .arg(
            Arg::new("gradient")
                .short('g')
                .long("gradient")
                .default_value("grayscale")
                .help("Gradient to use for the output"),
        )
        .arg(
            Arg::new("x_constant")
                .short('x')
                .long("x-constant")
                .default_value("-0.8")
                .help("Define a complex constant"),
        )
        .arg(
            Arg::new("y_constant")
                .short('y')
                .long("y-constant")
                .default_value("0.156")
                .help("Define a complex constant"),
        )
        .arg(
            Arg::new("zoom")
                .short('z')
                .long("zoom")
                .default_value("1.0")
                .help("Define a zoom/magnification to render at"),
        )
        .get_matches();

    let image_size = IVec2 {
        x: matches
            .get_one::<String>("width")
            .unwrap()
            .parse::<u64>()
            .unwrap(),
        y: matches
            .get_one::<String>("height")
            .unwrap()
            .parse::<u64>()
            .unwrap(),
    };

    let grayscale = colorgrad::CustomGradient::new()
        .colors(&[
            Color::from_rgba8(0, 0, 0, 255),
            Color::from_rgba8(255, 255, 255, 255),
        ])
        .build()
        .unwrap();

    let gradient: Gradient = match matches
        .get_one::<String>("gradient")
        .unwrap()
        .to_ascii_lowercase()
        .trim()
    {
        "grayscale" => grayscale,
        "rainbow" => colorgrad::rainbow(),
        "inferno" => colorgrad::inferno(),
        "viridis" => colorgrad::viridis(),
        _ => {
            println!("Please choose one of the following:\n - Grayscale\n - Rainbow\n - Inferno\n - Viridis");
            return;
        }
    };

    // let constant = Vec2 { x: -0.4, y: 0.6};
    // let constant = Vec2 { x: -0.8, y: 0.156 };
    let constant = Vec2 {
        x: matches.get_one::<String>("x_constant").unwrap().parse::<f64>().unwrap(),
        y: matches.get_one::<String>("y_constant").unwrap().parse::<f64>().unwrap(),
    };
    let iterations = matches.get_one::<String>("iterations").unwrap().parse::<usize>().unwrap();
    let zoom = matches.get_one::<String>("zoom").unwrap().parse::<f64>().unwrap();

    let fractal_type = match matches
        .get_one::<String>("fractal")
        .unwrap()
        .to_ascii_lowercase()
        .trim()
    {
        "julia" => Fractal::Julia,
        _ => {
            println!("Please choose one of the following:\n - julia\n - mandelbrot");
            return;
        }
    };

    let out_name: &str = matches.get_one::<String>("name").unwrap();

    let timer = Instant::now();
    render(
        image_size,
        constant,
        zoom,
        iterations,
        gradient,
        fractal_type,
        out_name,
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
fn iterate_to_max_julia(
    initial_z: Vec2,
    constant: Vec2,
    fractal_zoom: f64,
    max_iterations: usize,
) -> usize {
    let mut zn = Vec2 {
        x: initial_z.x / fractal_zoom,
        y: initial_z.y / fractal_zoom,
    };
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
    gradient: Gradient,
    fractal_type: Fractal,
    out_name: &str,
) {
    let buffer: RgbImage = ImageBuffer::new(
        render_size.x.try_into().unwrap(),
        render_size.y.try_into().unwrap(),
    );

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
