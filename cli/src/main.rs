use std::error::Error;
use std::time::Instant;

use image::RgbImage;
use tracer::{Camera, Point3, Sphere};

const IMAGE_WIDTH: u32 = 400;
const ASPECT_RATIO: f64 = 16.0 / 9.0;

fn main() -> Result<(), Box<dyn Error>> {
    let output = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "output.png".to_string());

    let camera = Camera::new(IMAGE_WIDTH, ASPECT_RATIO);
    let sphere = Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5);

    let start = Instant::now();
    let pixels = tracer::render(&camera, &sphere);
    let elapsed = start.elapsed();

    let bytes: Vec<u8> = pixels.into_iter().flat_map(tracer::to_rgb8).collect();
    let image = RgbImage::from_raw(camera.image_width, camera.image_height, bytes)
        .ok_or("pixel buffer doesn't match image size")?;
    image.save(&output)?;

    eprintln!(
        "Rendered {}x{} in {:.2?} -> {output}",
        camera.image_width, camera.image_height, elapsed
    );
    Ok(())
}
