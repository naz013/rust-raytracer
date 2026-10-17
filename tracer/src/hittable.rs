use std::ops::Range;

use crate::material::Material;
use crate::ray::Ray;
use crate::vec3::{Point3, Vec3};

/// Where a ray hit a surface.
///
/// `'a` ties the record to the object that was hit: `material` borrows that
/// object's material instead of copying it, so a record can't outlive the scene.
#[derive(Debug, Clone, Copy)]
pub struct HitRecord<'a> {
    pub point: Point3,
    /// Unit normal, always pointing against the incoming ray.
    pub normal: Vec3,
    pub t: f64,
    /// True if the ray hit the outside of the surface.
    pub front_face: bool,
    pub material: &'a Material,
}

impl<'a> HitRecord<'a> {
    /// Builds a record from the surface's outward unit normal, flipping it
    /// when the ray comes from inside.
    pub fn new(ray: &Ray, t: f64, outward_normal: Vec3, material: &'a Material) -> Self {
        let front_face = ray.direction.dot(outward_normal) < 0.0;
        let normal = if front_face {
            outward_normal
        } else {
            -outward_normal
        };
        Self {
            point: ray.at(t),
            normal,
            t,
            front_face,
            material,
        }
    }
}

/// Anything a ray can hit.
///
/// `Send + Sync` lets scenes be shared across rayon's worker threads.
pub trait Hittable: Send + Sync {
    /// The nearest hit with `t` inside `t_range`, if any.
    fn hit(&self, ray: &Ray, t_range: Range<f64>) -> Option<HitRecord<'_>>;
}

/// A collection of objects, itself hittable: reports the closest hit.
#[derive(Default)]
pub struct HittableList {
    objects: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, object: impl Hittable + 'static) {
        self.objects.push(Box::new(object));
    }
}

impl Hittable for HittableList {
    fn hit(&self, ray: &Ray, t_range: Range<f64>) -> Option<HitRecord<'_>> {
        let mut closest = None;
        let mut closest_t = t_range.end;
        for object in &self.objects {
            // Shrink the range as we go, so only nearer hits can win.
            if let Some(rec) = object.hit(ray, t_range.start..closest_t) {
                closest_t = rec.t;
                closest = Some(rec);
            }
        }
        closest
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sphere::Sphere;
    use crate::vec3::Color;

    #[test]
    fn list_reports_closest_hit() {
        let gray = Material::lambertian(Color::new(0.5, 0.5, 0.5));
        let mut list = HittableList::new();
        list.add(Sphere::new(Point3::new(0.0, 0.0, -5.0), 1.0, gray));
        list.add(Sphere::new(Point3::new(0.0, 0.0, -2.0), 0.5, gray));

        let ray = Ray::new(Point3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        let rec = list.hit(&ray, 0.0..f64::INFINITY).unwrap();
        assert_eq!(rec.t, 1.5);
    }

    #[test]
    fn empty_list_hits_nothing() {
        let ray = Ray::new(Point3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        assert!(HittableList::new().hit(&ray, 0.0..f64::INFINITY).is_none());
    }
}
