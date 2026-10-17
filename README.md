# raytracer

A CPU ray tracer in Rust, following Peter Shirley's
[_Ray Tracing in One Weekend_](https://raytracing.github.io/books/RayTracingInOneWeekend.html).

## Layout

| Crate    | What it is                                                        |
|----------|-------------------------------------------------------------------|
| `tracer` | Core library: math, camera, scene, `render`. No UI/platform code. |
| `cli`    | Renders a scene to a PNG.                                         |

## Build and run

Needs a stable Rust toolchain from [rustup.rs](https://rustup.rs).

```sh
cargo run --release -p cli            # writes output.png
cargo run --release -p cli -- sky.png # custom output path
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```
