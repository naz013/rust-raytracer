use std::ops::Range;

use crate::hittable::{HitRecord, Hittable};
use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::Point3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub material: Material,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, material: Material) -> Self {
        Self {
            center,
            radius: radius.max(0.0),
            material,
        }
    }
}

impl Hittable for Sphere {
    /// Solves `|origin + t*dir - center|² = r²`, a quadratic in `t`. Using
    /// `h = b / -2` keeps the arithmetic simpler than the textbook formula.
    fn hit(&self, ray: &Ray, t_range: Range<f64>) -> Option<HitRecord<'_>> {
        let oc = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            return None;
        }

        // Try the near root first; fall back to the far one (ray starts inside).
        let sqrt_d = discriminant.sqrt();
        let t = [(h - sqrt_d) / a, (h + sqrt_d) / a]
            .into_iter()
            .find(|t| t_range.contains(t))?;

        let outward_normal = (ray.at(t) - self.center) / self.radius;
        Some(HitRecord::new(ray, t, outward_normal, &self.material))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec3::{Color, Vec3};

    const ANY_T: Range<f64> = 0.0..f64::INFINITY;

    fn sphere_ahead() -> Sphere {
        let material = Material::lambertian(Color::new(0.5, 0.5, 0.5));
        Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5, material)
    }

    #[test]
    fn ray_through_center_hits_front_surface() {
        let sphere = sphere_ahead();
        let ray = Ray::new(Point3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        let rec = sphere.hit(&ray, ANY_T).unwrap();
        assert_eq!(rec.t, 0.5);
        assert_eq!(rec.point, Point3::new(0.0, 0.0, -0.5));
        assert_eq!(rec.normal, Vec3::new(0.0, 0.0, 1.0));
        assert!(rec.front_face);
    }

    #[test]
    fn ray_pointing_away_misses() {
        let sphere = sphere_ahead();
        let ray = Ray::new(Point3::ZERO, Vec3::new(0.0, 1.0, 0.0));
        assert!(sphere.hit(&ray, ANY_T).is_none());
    }

    #[test]
    fn tangent_ray_touches_once() {
        let sphere = sphere_ahead();
        let ray = Ray::new(Point3::new(0.0, 0.5, 0.0), Vec3::new(0.0, 0.0, -1.0));
        let rec = sphere.hit(&ray, ANY_T).unwrap();
        assert_eq!(rec.t, 1.0);
    }

    #[test]
    fn sphere_behind_ray_is_not_hit() {
        let sphere = sphere_ahead();
        let ray = Ray::new(Point3::ZERO, Vec3::new(0.0, 0.0, 1.0));
        assert!(sphere.hit(&ray, ANY_T).is_none());
    }

    #[test]
    fn hit_distance_scales_with_direction_length() {
        let sphere = sphere_ahead();
        // A non-unit direction should give the same hit point, at a smaller t.
        let ray = Ray::new(Point3::ZERO, Vec3::new(0.0, 0.0, -2.0));
        let rec = sphere.hit(&ray, ANY_T).unwrap();
        assert_eq!(rec.t, 0.25);
        assert_eq!(rec.point, Point3::new(0.0, 0.0, -0.5));
    }

    #[test]
    fn ray_from_inside_hits_back_face_with_flipped_normal() {
        let sphere = sphere_ahead();
        let ray = Ray::new(Point3::new(0.0, 0.0, -1.0), Vec3::new(0.0, 0.0, -1.0));
        let rec = sphere.hit(&ray, ANY_T).unwrap();
        assert_eq!(rec.t, 0.5);
        assert!(!rec.front_face);
        assert_eq!(rec.normal, Vec3::new(0.0, 0.0, 1.0));
    }

    #[test]
    fn hits_outside_t_range_are_ignored() {
        let sphere = sphere_ahead();
        let ray = Ray::new(Point3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        // Near root 0.5 is excluded, so the far root 1.5 is reported.
        assert_eq!(sphere.hit(&ray, 0.6..10.0).unwrap().t, 1.5);
        assert!(sphere.hit(&ray, 0.0..0.4).is_none());
    }
}
