use crate::vec3::{Point3, Vec3};

/// A half-line `origin + t * direction`. The direction isn't required to be unit length.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ray {
    pub origin: Point3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Point3, direction: Vec3) -> Self {
        Self { origin, direction }
    }

    /// The point at parameter `t` along the ray.
    pub fn at(&self, t: f64) -> Point3 {
        self.origin + t * self.direction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn at_walks_along_direction() {
        let ray = Ray::new(Point3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 2.0, 0.0));
        assert_eq!(ray.at(0.0), ray.origin);
        assert_eq!(ray.at(1.5), Point3::new(1.0, 3.0, 0.0));
    }
}
