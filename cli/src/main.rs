use std::error::Error;
use std::process::ExitCode;
use std::time::Instant;

use image::RgbImage;
use tracer::{Camera, RenderSettings, Scene, scene};

const USAGE: &str = "\
Usage: cli [options]

Options:
  --scene <name>   cover (default) or materials
  --width <px>     image width (default: the scene's own)
  --spp <n>        samples per pixel (default: 100)
  --depth <n>      max bounces per ray (default: 50)
  --seed <n>       random seed for the cover scene (default: 42)
  -o, --output     output PNG path (default: output.png)
  -h, --help       show this help";

struct Args {
    scene: String,
    width: Option<u32>,
    settings: RenderSettings,
    seed: u64,
    output: String,
}

fn main() -> ExitCode {
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(Some(args)) => args,
        Ok(None) => {
            println!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        Err(message) => {
            eprintln!("error: {message}\n\n{USAGE}");
            return ExitCode::FAILURE;
        }
    };

    match run(args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let Scene {
        world,
        camera: mut camera_config,
    } = match args.scene.as_str() {
        "cover" => scene::cover(args.seed),
        "materials" => scene::materials(),
        other => return Err(format!("unknown scene '{other}'").into()),
    };
    if let Some(width) = args.width {
        camera_config.image_width = width;
    }
    let camera = Camera::new(&camera_config);

    eprintln!(
        "Rendering '{}' at {}x{}, {} spp, depth {}...",
        args.scene,
        camera.image_width,
        camera.image_height,
        args.settings.samples_per_pixel,
        args.settings.max_depth
    );
    let start = Instant::now();
    let pixels = tracer::render(&camera, &world, &args.settings);
    let elapsed = start.elapsed();

    let bytes: Vec<u8> = pixels.into_iter().flat_map(tracer::to_rgb8).collect();
    let image = RgbImage::from_raw(camera.image_width, camera.image_height, bytes)
        .ok_or("pixel buffer doesn't match image size")?;
    image.save(&args.output)?;

    eprintln!("Done in {elapsed:.2?} -> {}", args.output);
    Ok(())
}

/// Parses command-line flags. `Ok(None)` means `--help` was requested.
fn parse_args(mut args: impl Iterator<Item = String>) -> Result<Option<Args>, String> {
    let mut parsed = Args {
        scene: "cover".to_string(),
        width: None,
        settings: RenderSettings::default(),
        seed: 42,
        output: "output.png".to_string(),
    };

    while let Some(flag) = args.next() {
        let mut value = || args.next().ok_or(format!("{flag} needs a value"));
        match flag.as_str() {
            "--scene" => parsed.scene = value()?,
            "--width" => parsed.width = Some(parse_number(&flag, value()?)?),
            "--spp" => parsed.settings.samples_per_pixel = parse_number(&flag, value()?)?,
            "--depth" => parsed.settings.max_depth = parse_number(&flag, value()?)?,
            "--seed" => parsed.seed = parse_number(&flag, value()?)?,
            "-o" | "--output" => parsed.output = value()?,
            "-h" | "--help" => return Ok(None),
            other => return Err(format!("unknown argument '{other}'")),
        }
    }
    Ok(Some(parsed))
}

/// Parses any number type; the target type is inferred from where the result goes.
fn parse_number<T: std::str::FromStr>(flag: &str, value: String) -> Result<T, String> {
    value
        .parse()
        .map_err(|_| format!("{flag} expects a number, got '{value}'"))
}
