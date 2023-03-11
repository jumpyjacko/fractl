use core::panic;

use clap::{Arg, Command};
use colorgrad::{Color, Gradient};

use crate::{IVec2, Vec2};

pub enum Fractal {
    Julia,
    Mandelbrot,
}

pub struct UserVars {
    pub image_size: IVec2,
    pub gradient: Gradient,
    pub constant: Vec2,
    pub iterations: usize,
    pub zoom: f64,
    pub fractal_type: Fractal,
    pub out_type: bool,
    pub out_name: String,
}

impl UserVars {
    pub fn new() -> UserVars {
        return setup_cli();
    }
}

fn setup_cli() -> UserVars {
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
            Arg::new("out_type")
                .long("output-type")
                .default_value("image")
                .help("Type of output (image or video)"),
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

    let inv_grayscale = colorgrad::CustomGradient::new()
        .colors(&[
            Color::from_rgba8(255, 255, 255, 255),
            Color::from_rgba8(0, 0, 0, 255),
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
        "inverted_grayscale" => inv_grayscale,
        "rainbow" => colorgrad::rainbow(),
        "inferno" => colorgrad::inferno(),
        "viridis" => colorgrad::viridis(),
        _ => {
            println!("Please choose one of the following:\n - Grayscale\n - Inverted_grayscale\n - Rainbow\n - Inferno\n - Viridis");
            panic!();
        }
    };

    let constant = Vec2 {
        x: matches
            .get_one::<String>("x_constant")
            .unwrap()
            .parse::<f64>()
            .unwrap(),
        y: matches
            .get_one::<String>("y_constant")
            .unwrap()
            .parse::<f64>()
            .unwrap(),
    };
    let iterations = matches
        .get_one::<String>("iterations")
        .unwrap()
        .parse::<usize>()
        .unwrap();
    let zoom = matches
        .get_one::<String>("zoom")
        .unwrap()
        .parse::<f64>()
        .unwrap();
    let out_type = match matches
        .get_one::<String>("out_type")
        .unwrap()
        .to_ascii_lowercase()
        .trim()
    {
        "image" => true,
        "video" => false,
        _ => {
            println!("Please choose either 'image' or 'video'");
            panic!();
        }
    };

    let fractal_type = match matches
        .get_one::<String>("fractal")
        .unwrap()
        .to_ascii_lowercase()
        .trim()
    {
        "julia" => Fractal::Julia,
        "mandelbrot" => Fractal::Mandelbrot,
        _ => {
            println!("Please choose one of the following:\n - julia\n - mandelbrot");
            panic!();
        }
    };

    let out_name = matches.get_one::<String>("name").unwrap();

    UserVars {
        image_size,
        gradient,
        constant,
        iterations,
        zoom,
        fractal_type,
        out_type,
        out_name: out_name.to_owned(),
    }
}
