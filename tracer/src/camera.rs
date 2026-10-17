use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// A fixed pinhole camera at the origin looking down -Z.
/// Phase 2 makes it positionable and adds defocus blur.
#[derive(Debug, Clone)]
pub struct Camera {
    pub image_width: u32,
    pub image_height: u32,
    center: Point3,
    pixel00: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
}

impl Camera {
    pub fn new(image_width: u32, aspect_ratio: f64) -> Self {
        let image_height = ((f64::from(image_width) / aspect_ratio) as u32).max(1);

        // The viewport is a rectangle in the scene that the image maps onto.
        // Its width uses the real pixel ratio, since image_height was rounded.
        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * f64::from(image_width) / f64::from(image_height);
        let center = Point3::ZERO;

        // u runs left→right across the viewport, v runs top→bottom.
        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);
        let pixel_delta_u = viewport_u / f64::from(image_width);
        let pixel_delta_v = viewport_v / f64::from(image_height);

        let viewport_upper_left =
            center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        let pixel00 = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

        Self {
            image_width,
            image_height,
            center,
            pixel00,
            pixel_delta_u,
            pixel_delta_v,
        }
    }

    /// The ray from the camera through the center of pixel `(i, j)`,
    /// where `(0, 0)` is the top-left pixel.
    pub fn get_ray(&self, i: u32, j: u32) -> Ray {
        let pixel_center =
            self.pixel00 + f64::from(i) * self.pixel_delta_u + f64::from(j) * self.pixel_delta_v;
        Ray::new(self.center, pixel_center - self.center)
    }
}
