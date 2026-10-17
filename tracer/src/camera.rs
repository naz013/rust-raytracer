use crate::ray::Ray;
use crate::rng::Rng;
use crate::vec3::{Point3, Vec3};

/// User-facing camera settings. `Camera::new` turns these into the vectors
/// used to generate rays.
#[derive(Debug, Clone, PartialEq)]
pub struct CameraConfig {
    pub image_width: u32,
    pub aspect_ratio: f64,
    /// Vertical field of view, in degrees.
    pub vfov: f64,
    pub look_from: Point3,
    pub look_at: Point3,
    /// Which way is "up" for the camera (needn't be exactly perpendicular).
    pub vup: Vec3,
    /// Cone angle of rays through each pixel, in degrees. 0 = everything sharp.
    pub defocus_angle: f64,
    /// Distance from `look_from` to the plane in perfect focus.
    pub focus_dist: f64,
}

impl Default for CameraConfig {
    fn default() -> Self {
        Self {
            image_width: 400,
            aspect_ratio: 16.0 / 9.0,
            vfov: 90.0,
            look_from: Point3::ZERO,
            look_at: Point3::new(0.0, 0.0, -1.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle: 0.0,
            focus_dist: 10.0,
        }
    }
}

/// A thin-lens camera: rays start on a disk around `look_from` (giving depth
/// of field) and pass through a random point inside each pixel (antialiasing).
#[derive(Debug, Clone)]
pub struct Camera {
    pub image_width: u32,
    pub image_height: u32,
    center: Point3,
    pixel00: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
    defocus: bool,
}

impl Camera {
    pub fn new(config: &CameraConfig) -> Self {
        let image_width = config.image_width.max(1);
        let image_height = ((f64::from(image_width) / config.aspect_ratio) as u32).max(1);

        // The viewport is a rectangle on the focus plane that the image maps onto.
        // Its width uses the real pixel ratio, since image_height was rounded.
        let h = (config.vfov.to_radians() / 2.0).tan();
        let viewport_height = 2.0 * h * config.focus_dist;
        let viewport_width = viewport_height * f64::from(image_width) / f64::from(image_height);

        // Orthonormal camera basis: w points backwards, u right, v up.
        let center = config.look_from;
        let w = (config.look_from - config.look_at).unit();
        let u = config.vup.cross(w).unit();
        let v = w.cross(u);

        // viewport_u runs left→right, viewport_v top→bottom.
        let viewport_u = viewport_width * u;
        let viewport_v = viewport_height * -v;
        let pixel_delta_u = viewport_u / f64::from(image_width);
        let pixel_delta_v = viewport_v / f64::from(image_height);

        let viewport_upper_left =
            center - config.focus_dist * w - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00 = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        let defocus_radius = config.focus_dist * (config.defocus_angle / 2.0).to_radians().tan();

        Self {
            image_width,
            image_height,
            center,
            pixel00,
            pixel_delta_u,
            pixel_delta_v,
            defocus_disk_u: defocus_radius * u,
            defocus_disk_v: defocus_radius * v,
            defocus: config.defocus_angle > 0.0,
        }
    }

    /// A ray through a random point inside pixel `(i, j)`, where `(0, 0)` is
    /// the top-left pixel.
    pub fn get_ray(&self, i: u32, j: u32, rng: &mut Rng) -> Ray {
        let offset_x = rng.f64() - 0.5;
        let offset_y = rng.f64() - 0.5;
        let pixel_sample = self.pixel00
            + (f64::from(i) + offset_x) * self.pixel_delta_u
            + (f64::from(j) + offset_y) * self.pixel_delta_v;

        let origin = if self.defocus {
            let p = rng.in_unit_disk();
            self.center + p.x * self.defocus_disk_u + p.y * self.defocus_disk_v
        } else {
            self.center
        };
        Ray::new(origin, pixel_sample - origin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn image_height_follows_aspect_ratio() {
        let camera = Camera::new(&CameraConfig {
            image_width: 400,
            aspect_ratio: 16.0 / 9.0,
            ..CameraConfig::default()
        });
        assert_eq!(camera.image_height, 225);
    }

    #[test]
    fn center_pixel_looks_at_target() {
        let config = CameraConfig {
            image_width: 101,
            aspect_ratio: 1.0,
            look_from: Point3::new(3.0, 2.0, 1.0),
            look_at: Point3::new(-1.0, 0.0, -2.0),
            ..CameraConfig::default()
        };
        let camera = Camera::new(&config);
        let forward = (config.look_at - config.look_from).unit();

        let mut rng = Rng::new(1);
        let ray = camera.get_ray(50, 50, &mut rng);
        assert_eq!(ray.origin, config.look_from);
        assert!(ray.direction.unit().dot(forward) > 0.9999);
    }

    #[test]
    fn defocus_spreads_ray_origins_but_keeps_focus_point() {
        let config = CameraConfig {
            image_width: 1,
            aspect_ratio: 1.0,
            defocus_angle: 10.0,
            focus_dist: 5.0,
            ..CameraConfig::default()
        };
        let camera = Camera::new(&config);
        let mut rng = Rng::new(1);

        let radius = 5.0 * 5f64.to_radians().tan();
        for _ in 0..100 {
            let ray = camera.get_ray(0, 0, &mut rng);
            assert!(ray.origin.length() <= radius);
            assert_eq!(ray.origin.z, 0.0);
            // Every ray still passes through the single pixel on the focus plane.
            let on_plane = ray.at(5.0 / -ray.direction.z);
            assert!(on_plane.x.abs() <= 5.0 && on_plane.y.abs() <= 5.0);
        }
    }
}
