use crate::vec3::Vec3;

/// A small, fast, seedable pseudo-random generator (SplitMix64).
///
/// Not cryptographic; it only needs to be fast and well distributed. Giving
/// each image row its own seeded `Rng` makes renders reproducible no matter
/// how rayon schedules the rows across threads.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in `[0, 1)`.
    pub fn f64(&mut self) -> f64 {
        // The top 53 bits fill an f64 mantissa exactly.
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform in `[min, max)`.
    pub fn range(&mut self, min: f64, max: f64) -> f64 {
        min + (max - min) * self.f64()
    }

    /// A vector with each component uniform in `[min, max)`.
    pub fn vec3(&mut self, min: f64, max: f64) -> Vec3 {
        Vec3::new(
            self.range(min, max),
            self.range(min, max),
            self.range(min, max),
        )
    }

    /// A uniformly distributed direction (a point on the unit sphere).
    pub fn unit_vector(&mut self) -> Vec3 {
        // Rejection sampling: pick points in the cube until one lands inside
        // the sphere, then push it out to the surface. Tiny vectors are
        // rejected too, since normalizing them would blow up.
        loop {
            let p = self.vec3(-1.0, 1.0);
            let len_sq = p.length_squared();
            if 1e-160 < len_sq && len_sq <= 1.0 {
                return p / len_sq.sqrt();
            }
        }
    }

    /// A uniformly distributed point inside the unit disk on the z = 0 plane.
    pub fn in_unit_disk(&mut self) -> Vec3 {
        loop {
            let p = Vec3::new(self.range(-1.0, 1.0), self.range(-1.0, 1.0), 0.0);
            if p.length_squared() < 1.0 {
                return p;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_sequence() {
        let mut a = Rng::new(42);
        let mut b = Rng::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn f64_stays_in_unit_interval() {
        let mut rng = Rng::new(1);
        for _ in 0..10_000 {
            let x = rng.f64();
            assert!((0.0..1.0).contains(&x));
        }
    }

    #[test]
    fn f64_mean_is_about_half() {
        let mut rng = Rng::new(7);
        let n = 100_000;
        let mean = (0..n).map(|_| rng.f64()).sum::<f64>() / f64::from(n);
        assert!((mean - 0.5).abs() < 0.01, "mean was {mean}");
    }

    #[test]
    fn unit_vectors_have_length_one() {
        let mut rng = Rng::new(3);
        for _ in 0..1000 {
            assert!((rng.unit_vector().length() - 1.0).abs() < 1e-12);
        }
    }

    #[test]
    fn disk_points_are_inside_the_disk() {
        let mut rng = Rng::new(5);
        for _ in 0..1000 {
            let p = rng.in_unit_disk();
            assert!(p.length() < 1.0 && p.z == 0.0);
        }
    }
}
