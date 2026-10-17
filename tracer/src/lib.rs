//! A CPU ray tracer based on "Ray Tracing in One Weekend".
//!
//! Pure Rust with no UI or platform code: front ends (the CLI, the egui app)
//! call [`render`] and turn the resulting colors into pixels.

pub mod camera;
pub mod ray;
pub mod sphere;
pub mod vec3;

pub use camera::Camera;
pub use ray::Ray;
pub use sphere::Sphere;
pub use vec3::{Color, Point3, Vec3};

/// The color seen along `ray`: red where it hits the sphere, sky elsewhere.
pub fn ray_color(ray: &Ray, sphere: &Sphere) -> Color {
    if sphere.hit(ray).is_some() {
        return Color::new(1.0, 0.0, 0.0);
    }

    // Blend white → sky blue based on how far up the ray points.
    let a = 0.5 * (ray.direction.unit().y + 1.0);
    Color::ONE.lerp(Color::new(0.5, 0.7, 1.0), a)
}

/// Renders the scene to a row-major list of colors, top row first.
pub fn render(camera: &Camera, sphere: &Sphere) -> Vec<Color> {
    (0..camera.image_height)
        .flat_map(|j| (0..camera.image_width).map(move |i| (i, j)))
        .map(|(i, j)| ray_color(&camera.get_ray(i, j), sphere))
        .collect()
}

/// Converts a linear color to 8-bit RGB, clamping each channel to `[0, 1)`.
pub fn to_rgb8(color: Color) -> [u8; 3] {
    let channel = |c: f64| (256.0 * c.clamp(0.0, 0.999)) as u8;
    [channel(color.x), channel(color.y), channel(color.z)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_rgb8_clamps_and_scales() {
        assert_eq!(to_rgb8(Color::ZERO), [0, 0, 0]);
        assert_eq!(to_rgb8(Color::ONE), [255, 255, 255]);
        assert_eq!(to_rgb8(Color::new(0.5, -1.0, 2.0)), [128, 0, 255]);
    }

    #[test]
    fn render_produces_one_color_per_pixel() {
        let camera = Camera::new(16, 2.0);
        let sphere = Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5);
        let pixels = render(&camera, &sphere);
        assert_eq!(pixels.len(), 16 * 8);
    }
}
