use rayon::prelude::*;

use crate::camera::Camera;
use crate::hittable::Hittable;
use crate::ray::Ray;
use crate::rng::Rng;
use crate::vec3::Color;

/// Quality knobs that don't change what the camera sees.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderSettings {
    /// Random rays averaged per pixel; more means less noise.
    pub samples_per_pixel: u32,
    /// Maximum bounces per ray before it is treated as absorbed.
    pub max_depth: u32,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            samples_per_pixel: 100,
            max_depth: 50,
        }
    }
}

/// Renders `world` to a row-major list of linear colors, top row first.
///
/// Rows are rendered in parallel. Each row seeds its own `Rng` from its index,
/// so the image is identical however the rows are spread over threads.
pub fn render(camera: &Camera, world: &dyn Hittable, settings: &RenderSettings) -> Vec<Color> {
    let width = camera.image_width as usize;
    let samples = settings.samples_per_pixel.max(1);

    let mut pixels = vec![Color::ZERO; width * camera.image_height as usize];
    // `par_chunks_mut` hands each thread its own row as a `&mut [Color]`.
    // The rows don't overlap, so the borrow checker allows mutating them in parallel.
    pixels
        .par_chunks_mut(width)
        .enumerate()
        .for_each(|(j, row)| {
            let j = j as u32;
            let mut rng = Rng::new(u64::from(j));
            for (pixel, i) in row.iter_mut().zip(0u32..) {
                let mut sum = Color::ZERO;
                for _ in 0..samples {
                    let ray = camera.get_ray(i, j, &mut rng);
                    sum += ray_color(&ray, world, settings.max_depth, &mut rng);
                }
                *pixel = sum / f64::from(samples);
            }
        });
    pixels
}

/// The light arriving back along `ray`, following up to `depth` bounces.
pub fn ray_color(ray: &Ray, world: &dyn Hittable, depth: u32, rng: &mut Rng) -> Color {
    if depth == 0 {
        return Color::ZERO;
    }

    // Start slightly above 0 so a bounced ray doesn't re-hit its own surface
    // because of floating-point error ("shadow acne").
    let Some(hit) = world.hit(ray, 0.001..f64::INFINITY) else {
        return sky(ray);
    };

    match hit.material.scatter(ray, &hit, rng) {
        Some(scatter) => scatter.attenuation * ray_color(&scatter.ray, world, depth - 1, rng),
        None => Color::ZERO,
    }
}

/// Background: white at the horizon blending to blue overhead.
fn sky(ray: &Ray) -> Color {
    let a = 0.5 * (ray.direction.unit().y + 1.0);
    Color::ONE.lerp(Color::new(0.5, 0.7, 1.0), a)
}

/// Converts a linear color to 8-bit sRGB-ish RGB (gamma 2), clamping each
/// channel to `[0, 1)`.
pub fn to_rgb8(color: Color) -> [u8; 3] {
    let channel = |c: f64| (256.0 * c.max(0.0).sqrt().clamp(0.0, 0.999)) as u8;
    [channel(color.x), channel(color.y), channel(color.z)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::CameraConfig;
    use crate::hittable::HittableList;
    use crate::material::Material;
    use crate::sphere::Sphere;
    use crate::vec3::Point3;

    fn tiny_scene() -> (Camera, HittableList) {
        let camera = Camera::new(&CameraConfig {
            image_width: 16,
            aspect_ratio: 2.0,
            ..CameraConfig::default()
        });
        let mut world = HittableList::new();
        world.add(Sphere::new(
            Point3::new(0.0, 0.0, -1.0),
            0.5,
            Material::lambertian(Color::new(0.5, 0.5, 0.5)),
        ));
        (camera, world)
    }

    #[test]
    fn to_rgb8_applies_gamma_and_clamps() {
        assert_eq!(to_rgb8(Color::ZERO), [0, 0, 0]);
        assert_eq!(to_rgb8(Color::ONE), [255, 255, 255]);
        assert_eq!(to_rgb8(Color::new(0.25, -1.0, 2.0)), [128, 0, 255]);
    }

    #[test]
    fn render_produces_one_color_per_pixel() {
        let (camera, world) = tiny_scene();
        let settings = RenderSettings {
            samples_per_pixel: 2,
            max_depth: 5,
        };
        assert_eq!(render(&camera, &world, &settings).len(), 16 * 8);
    }

    #[test]
    fn render_is_deterministic() {
        let (camera, world) = tiny_scene();
        let settings = RenderSettings {
            samples_per_pixel: 4,
            max_depth: 5,
        };
        assert_eq!(
            render(&camera, &world, &settings),
            render(&camera, &world, &settings)
        );
    }

    #[test]
    fn zero_depth_is_black() {
        let ray = Ray::new(Point3::ZERO, Point3::new(0.0, 1.0, 0.0));
        let world = HittableList::new();
        assert_eq!(ray_color(&ray, &world, 0, &mut Rng::new(1)), Color::ZERO);
    }
}
