//! A CPU ray tracer based on "Ray Tracing in One Weekend".
//!
//! Pure Rust with no UI or platform code: front ends (the CLI, the egui app)
//! call [`render`] and turn the resulting colors into pixels with [`to_rgb8`].

pub mod camera;
pub mod hittable;
pub mod material;
pub mod ray;
pub mod render;
pub mod rng;
pub mod scene;
pub mod sphere;
pub mod vec3;

pub use camera::{Camera, CameraConfig};
pub use hittable::{HitRecord, Hittable, HittableList};
pub use material::Material;
pub use ray::Ray;
pub use render::{RenderSettings, render, render_pass, to_rgb8};
pub use rng::Rng;
pub use scene::Scene;
pub use sphere::Sphere;
pub use vec3::{Color, Point3, Vec3};
