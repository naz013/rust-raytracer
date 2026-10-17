use crate::ray::Ray;
use crate::vec3::Point3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
}

impl Sphere {
    pub fn new(center: Point3, radius: f64) -> Self {
        Self { center, radius }
    }

    /// The nearest `t` where `ray` enters the sphere, or `None` if it misses
    /// or that point is behind the ray's origin.
    ///
    /// Solves `|origin + t*dir - center|² = r²`, a quadratic in `t`. Using
    /// `h = b / -2` keeps the arithmetic simpler than the textbook formula.
    pub fn hit(&self, ray: &Ray) -> Option<f64> {
        let oc = self.center - ray.origin;
        let a = ray.direction.length_squared();
        let h = ray.direction.dot(oc);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = h * h - a * c;
        if discriminant < 0.0 {
            return None;
        }

        let t = (h - discriminant.sqrt()) / a;
        (t > 0.0).then_some(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec3::Vec3;

    fn sphere_ahead() -> Sphere {
        Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)
    }

    #[test]
    fn ray_through_center_hits_front_surface() {
        let ray = Ray::new(Point3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(sphere_ahead().hit(&ray), Some(0.5));
    }

    #[test]
    fn ray_pointing_away_misses() {
        let ray = Ray::new(Point3::ZERO, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(sphere_ahead().hit(&ray), None);
    }

    #[test]
    fn tangent_ray_touches_once() {
        let ray = Ray::new(Point3::new(0.0, 0.5, 0.0), Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(sphere_ahead().hit(&ray), Some(1.0));
    }

    #[test]
    fn sphere_behind_ray_is_not_hit() {
        let ray = Ray::new(Point3::ZERO, Vec3::new(0.0, 0.0, 1.0));
        assert_eq!(sphere_ahead().hit(&ray), None);
    }

    #[test]
    fn hit_distance_scales_with_direction_length() {
        // A non-unit direction should give the same hit point, at a smaller t.
        let ray = Ray::new(Point3::ZERO, Vec3::new(0.0, 0.0, -2.0));
        let t = sphere_ahead().hit(&ray).unwrap();
        assert_eq!(t, 0.25);
        assert_eq!(ray.at(t), Point3::new(0.0, 0.0, -0.5));
    }
}
