//! Ready-made scenes from the book.

use crate::camera::CameraConfig;
use crate::hittable::HittableList;
use crate::material::Material;
use crate::rng::Rng;
use crate::sphere::Sphere;
use crate::vec3::{Color, Point3};

/// Objects to render plus a camera that frames them well.
pub struct Scene {
    pub world: HittableList,
    pub camera: CameraConfig,
}

/// Three spheres (matte, hollow glass, fuzzy metal) on a yellow ground,
/// with shallow depth of field — the book's chapter 13 image.
pub fn materials() -> Scene {
    let ground = Material::lambertian(Color::new(0.8, 0.8, 0.0));
    let center = Material::lambertian(Color::new(0.1, 0.2, 0.5));
    let glass = Material::dielectric(1.5);
    let air_bubble = Material::dielectric(1.0 / 1.5);
    let gold = Material::metal(Color::new(0.8, 0.6, 0.2), 1.0);

    let mut world = HittableList::new();
    world.add(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0, ground));
    world.add(Sphere::new(Point3::new(0.0, 0.0, -1.2), 0.5, center));
    world.add(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.5, glass));
    world.add(Sphere::new(Point3::new(-1.0, 0.0, -1.0), 0.4, air_bubble));
    world.add(Sphere::new(Point3::new(1.0, 0.0, -1.0), 0.5, gold));

    Scene {
        world,
        camera: CameraConfig {
            vfov: 20.0,
            look_from: Point3::new(-2.0, 2.0, 1.0),
            look_at: Point3::new(0.0, 0.0, -1.0),
            defocus_angle: 10.0,
            focus_dist: 3.4,
            ..CameraConfig::default()
        },
    }
}

/// The book's cover: a field of small random spheres around three big ones.
/// The same `seed` always produces the same scene.
pub fn cover(seed: u64) -> Scene {
    let mut rng = Rng::new(seed);
    let mut world = HittableList::new();

    let ground = Material::lambertian(Color::new(0.5, 0.5, 0.5));
    world.add(Sphere::new(Point3::new(0.0, -1000.0, 0.0), 1000.0, ground));

    let big_metal_center = Point3::new(4.0, 0.2, 0.0);
    for a in -11..11 {
        for b in -11..11 {
            let center = Point3::new(
                f64::from(a) + 0.9 * rng.f64(),
                0.2,
                f64::from(b) + 0.9 * rng.f64(),
            );
            // Leave room around the big metal sphere.
            if (center - big_metal_center).length() <= 0.9 {
                continue;
            }

            let material = match rng.f64() {
                p if p < 0.8 => Material::lambertian(rng.vec3(0.0, 1.0) * rng.vec3(0.0, 1.0)),
                p if p < 0.95 => Material::metal(rng.vec3(0.5, 1.0), rng.range(0.0, 0.5)),
                _ => Material::dielectric(1.5),
            };
            world.add(Sphere::new(center, 0.2, material));
        }
    }

    let big_spheres = [
        (Point3::new(0.0, 1.0, 0.0), Material::dielectric(1.5)),
        (
            Point3::new(-4.0, 1.0, 0.0),
            Material::lambertian(Color::new(0.4, 0.2, 0.1)),
        ),
        (
            Point3::new(4.0, 1.0, 0.0),
            Material::metal(Color::new(0.7, 0.6, 0.5), 0.0),
        ),
    ];
    for (center, material) in big_spheres {
        world.add(Sphere::new(center, 1.0, material));
    }

    Scene {
        world,
        camera: CameraConfig {
            image_width: 1200,
            vfov: 20.0,
            look_from: Point3::new(13.0, 2.0, 3.0),
            look_at: Point3::ZERO,
            defocus_angle: 0.6,
            focus_dist: 10.0,
            ..CameraConfig::default()
        },
    }
}
