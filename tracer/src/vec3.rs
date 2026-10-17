use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub};

/// A 3D vector of `f64`s, used for points, directions and colors alike.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// A position in space. Same type as `Vec3`; the alias documents intent.
pub type Point3 = Vec3;

/// Linear RGB, each channel nominally in `[0, 1]`.
pub type Color = Vec3;

impl Vec3 {
    pub const ZERO: Self = Self::new(0.0, 0.0, 0.0);
    pub const ONE: Self = Self::new(1.0, 1.0, 1.0);

    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub fn dot(self, other: Self) -> f64 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(self, other: Self) -> Self {
        Self::new(
            self.y * other.z - self.z * other.y,
            self.z * other.x - self.x * other.z,
            self.x * other.y - self.y * other.x,
        )
    }

    pub fn length_squared(self) -> f64 {
        self.dot(self)
    }

    pub fn length(self) -> f64 {
        self.length_squared().sqrt()
    }

    /// This vector scaled to length 1.
    pub fn unit(self) -> Self {
        self / self.length()
    }

    /// Linear interpolation: `self` at `t = 0`, `other` at `t = 1`.
    pub fn lerp(self, other: Self, t: f64) -> Self {
        (1.0 - t) * self + t * other
    }

    /// True if every component is close to zero.
    pub fn near_zero(self) -> bool {
        const EPS: f64 = 1e-8;
        self.x.abs() < EPS && self.y.abs() < EPS && self.z.abs() < EPS
    }

    /// Mirror reflection of `self` about the surface normal `n` (unit length).
    pub fn reflect(self, n: Self) -> Self {
        self - 2.0 * self.dot(n) * n
    }

    /// Refraction of the unit vector `self` through a surface with unit normal `n`
    /// (pointing against `self`), by Snell's law with `eta_ratio = η_in / η_out`.
    pub fn refract(self, n: Self, eta_ratio: f64) -> Self {
        let cos_theta = (-self).dot(n).min(1.0);
        let r_out_perp = eta_ratio * (self + cos_theta * n);
        let r_out_parallel = -(1.0 - r_out_perp.length_squared()).abs().sqrt() * n;
        r_out_perp + r_out_parallel
    }
}

impl Add for Vec3 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for Vec3 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl Neg for Vec3 {
    type Output = Self;
    fn neg(self) -> Self {
        Self::new(-self.x, -self.y, -self.z)
    }
}

/// Component-wise product (used to tint colors).
impl Mul for Vec3 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Mul<f64> for Vec3 {
    type Output = Self;
    fn mul(self, t: f64) -> Self {
        Self::new(self.x * t, self.y * t, self.z * t)
    }
}

impl Mul<Vec3> for f64 {
    type Output = Vec3;
    fn mul(self, v: Vec3) -> Vec3 {
        v * self
    }
}

impl Div<f64> for Vec3 {
    type Output = Self;
    fn div(self, t: f64) -> Self {
        self * (1.0 / t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approx_eq(a: f64, b: f64) -> bool {
        (a - b).abs() < 1e-12
    }

    #[test]
    fn arithmetic() {
        let a = Vec3::new(1.0, 2.0, 3.0);
        let b = Vec3::new(4.0, 5.0, 6.0);
        assert_eq!(a + b, Vec3::new(5.0, 7.0, 9.0));
        assert_eq!(b - a, Vec3::new(3.0, 3.0, 3.0));
        assert_eq!(-a, Vec3::new(-1.0, -2.0, -3.0));
        assert_eq!(a * b, Vec3::new(4.0, 10.0, 18.0));
        assert_eq!(a * 2.0, Vec3::new(2.0, 4.0, 6.0));
        assert_eq!(2.0 * a, a * 2.0);
        assert_eq!(b / 2.0, Vec3::new(2.0, 2.5, 3.0));

        let mut c = a;
        c += b;
        assert_eq!(c, a + b);
    }

    #[test]
    fn dot_and_cross() {
        let x = Vec3::new(1.0, 0.0, 0.0);
        let y = Vec3::new(0.0, 1.0, 0.0);
        let z = Vec3::new(0.0, 0.0, 1.0);
        assert_eq!(x.dot(y), 0.0);
        assert_eq!(Vec3::new(1.0, 2.0, 3.0).dot(Vec3::new(4.0, 5.0, 6.0)), 32.0);
        assert_eq!(x.cross(y), z);
        assert_eq!(y.cross(x), -z);
    }

    #[test]
    fn length_and_unit() {
        let v = Vec3::new(3.0, 4.0, 0.0);
        assert_eq!(v.length_squared(), 25.0);
        assert_eq!(v.length(), 5.0);
        assert!(approx_eq(v.unit().length(), 1.0));
        let u = v.unit();
        assert!(approx_eq(u.x, 0.6) && approx_eq(u.y, 0.8) && u.z == 0.0);
    }

    #[test]
    fn lerp_endpoints_and_midpoint() {
        let a = Vec3::ZERO;
        let b = Vec3::new(2.0, 4.0, 6.0);
        assert_eq!(a.lerp(b, 0.0), a);
        assert_eq!(a.lerp(b, 1.0), b);
        assert_eq!(a.lerp(b, 0.5), Vec3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn near_zero() {
        assert!(Vec3::new(1e-9, -1e-9, 0.0).near_zero());
        assert!(!Vec3::new(1e-9, 1e-3, 0.0).near_zero());
    }

    #[test]
    fn reflect_flips_normal_component() {
        let n = Vec3::new(0.0, 1.0, 0.0);
        assert_eq!(
            Vec3::new(1.0, -1.0, 0.0).reflect(n),
            Vec3::new(1.0, 1.0, 0.0)
        );
    }

    #[test]
    fn refract_at_normal_incidence_goes_straight() {
        let n = Vec3::new(0.0, 1.0, 0.0);
        let down = Vec3::new(0.0, -1.0, 0.0);
        assert_eq!(down.refract(n, 1.0 / 1.5), down);
    }

    #[test]
    fn refract_with_equal_indices_does_not_bend() {
        let n = Vec3::new(0.0, 1.0, 0.0);
        let v = Vec3::new(1.0, -1.0, 0.0).unit();
        let r = v.refract(n, 1.0);
        assert!((r - v).length() < 1e-12);
    }

    #[test]
    fn refract_obeys_snells_law() {
        // Air (1.0) into glass (1.5) at 45°: sin(out) = sin(in) / 1.5.
        let n = Vec3::new(0.0, 1.0, 0.0);
        let v = Vec3::new(1.0, -1.0, 0.0).unit();
        let r = v.refract(n, 1.0 / 1.5);

        let sin_in = v.x;
        let sin_out = r.x / r.length();
        assert!(approx_eq(sin_out, sin_in / 1.5));
        assert!(approx_eq(r.length(), 1.0));
        assert!(
            r.y < 0.0,
            "refracted ray should keep going into the surface"
        );
    }
}
