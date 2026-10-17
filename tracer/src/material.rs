use crate::hittable::HitRecord;
use crate::ray::Ray;
use crate::rng::Rng;
use crate::vec3::Color;

/// How a surface scatters light.
///
/// A closed set of variants, so an enum fits better than a trait: `scatter`
/// matches on it, and the compiler checks that every variant is handled.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Material {
    /// Matte: scatters in random directions, biased toward the normal.
    Lambertian { albedo: Color },
    /// Mirror-like; `fuzz` in `[0, 1]` blurs the reflection.
    Metal { albedo: Color, fuzz: f64 },
    /// Glass, water, etc.: refracts when it can, reflects otherwise.
    Dielectric { refraction_index: f64 },
}

/// A scattered ray and how much it tints the light it carries back.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Scatter {
    pub attenuation: Color,
    pub ray: Ray,
}

impl Material {
    pub fn lambertian(albedo: Color) -> Self {
        Self::Lambertian { albedo }
    }

    pub fn metal(albedo: Color, fuzz: f64) -> Self {
        Self::Metal {
            albedo,
            fuzz: fuzz.clamp(0.0, 1.0),
        }
    }

    pub fn dielectric(refraction_index: f64) -> Self {
        Self::Dielectric { refraction_index }
    }

    /// Scatters `ray_in` at `hit`, or returns `None` if the light is absorbed.
    pub fn scatter(&self, ray_in: &Ray, hit: &HitRecord, rng: &mut Rng) -> Option<Scatter> {
        match *self {
            Self::Lambertian { albedo } => {
                let mut direction = hit.normal + rng.unit_vector();
                // The random vector can almost exactly cancel the normal.
                if direction.near_zero() {
                    direction = hit.normal;
                }
                Some(Scatter {
                    attenuation: albedo,
                    ray: Ray::new(hit.point, direction),
                })
            }

            Self::Metal { albedo, fuzz } => {
                let reflected = ray_in.direction.reflect(hit.normal).unit();
                let direction = reflected + fuzz * rng.unit_vector();
                // Fuzz can push the ray below the surface; absorb it then.
                (direction.dot(hit.normal) > 0.0).then(|| Scatter {
                    attenuation: albedo,
                    ray: Ray::new(hit.point, direction),
                })
            }

            Self::Dielectric { refraction_index } => {
                let eta_ratio = if hit.front_face {
                    1.0 / refraction_index
                } else {
                    refraction_index
                };

                let unit_direction = ray_in.direction.unit();
                let cos_theta = (-unit_direction).dot(hit.normal).min(1.0);
                let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

                let total_internal_reflection = eta_ratio * sin_theta > 1.0;
                let direction =
                    if total_internal_reflection || reflectance(cos_theta, eta_ratio) > rng.f64() {
                        unit_direction.reflect(hit.normal)
                    } else {
                        unit_direction.refract(hit.normal, eta_ratio)
                    };

                Some(Scatter {
                    attenuation: Color::ONE,
                    ray: Ray::new(hit.point, direction),
                })
            }
        }
    }
}

/// Schlick's approximation of how much light glass reflects at a given angle.
fn reflectance(cosine: f64, eta_ratio: f64) -> f64 {
    let r0 = ((1.0 - eta_ratio) / (1.0 + eta_ratio)).powi(2);
    r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::vec3::{Point3, Vec3};

    const UP: Vec3 = Vec3::new(0.0, 1.0, 0.0);

    fn hit_on_floor<'a>(ray: &Ray, material: &'a Material) -> HitRecord<'a> {
        // A hit on the plane y = 0 at the origin, normal +y.
        let t = -ray.origin.y / ray.direction.y;
        HitRecord::new(ray, t, UP, material)
    }

    #[test]
    fn reflectance_at_normal_incidence_matches_fresnel() {
        // Air → glass head-on reflects ((1 - 1.5) / (1 + 1.5))² = 4%.
        assert!((reflectance(1.0, 1.5) - 0.04).abs() < 1e-12);
        // Grazing angles reflect everything.
        assert!((reflectance(0.0, 1.5) - 1.0).abs() < 1e-12);
    }

    #[test]
    fn lambertian_scatters_away_from_surface() {
        let material = Material::lambertian(Color::new(0.5, 0.5, 0.5));
        let ray = Ray::new(Point3::new(0.0, 1.0, 0.0), -UP);
        let hit = hit_on_floor(&ray, &material);
        let mut rng = Rng::new(1);
        for _ in 0..1000 {
            let s = material.scatter(&ray, &hit, &mut rng).unwrap();
            assert!(s.ray.direction.dot(hit.normal) >= 0.0);
            assert_eq!(s.attenuation, Color::new(0.5, 0.5, 0.5));
        }
    }

    #[test]
    fn polished_metal_is_a_perfect_mirror() {
        let material = Material::metal(Color::ONE, 0.0);
        let ray = Ray::new(Point3::new(-1.0, 1.0, 0.0), Vec3::new(1.0, -1.0, 0.0));
        let hit = hit_on_floor(&ray, &material);
        let s = material.scatter(&ray, &hit, &mut Rng::new(1)).unwrap();
        assert!((s.ray.direction - Vec3::new(1.0, 1.0, 0.0).unit()).length() < 1e-12);
        assert_eq!(s.ray.origin, Point3::ZERO);
    }

    #[test]
    fn metal_fuzz_is_clamped() {
        assert_eq!(
            Material::metal(Color::ONE, 5.0),
            Material::Metal {
                albedo: Color::ONE,
                fuzz: 1.0
            }
        );
    }

    #[test]
    fn glass_totally_reflects_at_grazing_angle_from_inside() {
        // Leaving glass (back face) at a shallow angle: sin θ · 1.5 > 1, so
        // every sample must reflect back into the glass.
        let material = Material::dielectric(1.5);
        let ray = Ray::new(Point3::new(-1.0, -0.2, 0.0), Vec3::new(1.0, 0.2, 0.0));
        let hit = hit_on_floor(&ray, &material);
        assert!(!hit.front_face);

        let mut rng = Rng::new(1);
        for _ in 0..100 {
            let s = material.scatter(&ray, &hit, &mut rng).unwrap();
            assert!(s.ray.direction.y < 0.0, "ray escaped the glass");
        }
    }

    #[test]
    fn glass_mostly_refracts_head_on() {
        // Straight down onto glass: ~96% of samples pass straight through.
        let material = Material::dielectric(1.5);
        let ray = Ray::new(Point3::new(0.0, 1.0, 0.0), -UP);
        let hit = hit_on_floor(&ray, &material);

        let mut rng = Rng::new(1);
        let n = 10_000;
        let refracted = (0..n)
            .filter(|_| {
                material
                    .scatter(&ray, &hit, &mut rng)
                    .unwrap()
                    .ray
                    .direction
                    == -UP
            })
            .count();
        let fraction = refracted as f64 / f64::from(n);
        assert!((fraction - 0.96).abs() < 0.01, "refracted {fraction}");
    }
}
