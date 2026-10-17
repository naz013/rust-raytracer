//! Progressive rendering on a background thread.

use std::sync::mpsc::{self, Receiver};
use std::thread;

use tracer::{Camera, CameraConfig, Color, RenderSettings, Scene, scene};

/// The scenes the app can show.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SceneChoice {
    Cover,
    Materials,
}

impl SceneChoice {
    pub const ALL: [Self; 2] = [Self::Cover, Self::Materials];

    pub fn label(self) -> &'static str {
        match self {
            Self::Cover => "Book cover",
            Self::Materials => "Materials",
        }
    }

    pub fn build(self) -> Scene {
        match self {
            Self::Cover => scene::cover(42),
            Self::Materials => scene::materials(),
        }
    }
}

/// Everything needed to produce an image.
#[derive(Debug, Clone, PartialEq)]
pub struct Job {
    pub scene: SceneChoice,
    pub camera: CameraConfig,
    pub settings: RenderSettings,
}

/// The running average after some number of passes, as 8-bit RGB.
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<u8>,
    pub passes: u32,
}

/// Sums single-sample passes so the image can be shown as their average.
pub struct Accumulator {
    sum: Vec<Color>,
    passes: u32,
}

impl Accumulator {
    pub fn new(pixel_count: usize) -> Self {
        Self {
            sum: vec![Color::ZERO; pixel_count],
            passes: 0,
        }
    }

    pub fn add(&mut self, pass: &[Color]) {
        for (sum, color) in self.sum.iter_mut().zip(pass) {
            *sum += *color;
        }
        self.passes += 1;
    }

    pub fn passes(&self) -> u32 {
        self.passes
    }

    pub fn to_rgb8(&self) -> Vec<u8> {
        let scale = 1.0 / f64::from(self.passes.max(1));
        self.sum
            .iter()
            .flat_map(|&color| tracer::to_rgb8(color * scale))
            .collect()
    }
}

/// A render running on a background thread, sending a new `Frame` after
/// every pass.
///
/// Dropping the handle cancels the render: the thread notices that nobody is
/// listening when it tries to send its next frame, and exits.
pub struct RenderHandle {
    frames: Receiver<Frame>,
}

impl RenderHandle {
    /// Starts rendering `job`. `on_frame` runs on the worker thread after each
    /// new frame, e.g. to wake up the UI.
    pub fn start(job: Job, on_frame: impl Fn() + Send + 'static) -> Self {
        let (sender, frames) = mpsc::channel();

        // `move` transfers ownership of `job`, `sender` and `on_frame` into the
        // thread, like capturing vals in a Kotlin lambda, except that the
        // compiler then stops this function from touching them again.
        thread::spawn(move || {
            let world = job.scene.build().world;
            let camera = Camera::new(&job.camera);
            let pixel_count = camera.image_width as usize * camera.image_height as usize;
            let mut accumulator = Accumulator::new(pixel_count);

            for pass in 0..job.settings.samples_per_pixel.max(1) {
                let colors = tracer::render_pass(&camera, &world, job.settings.max_depth, pass);
                accumulator.add(&colors);

                let frame = Frame {
                    width: camera.image_width,
                    height: camera.image_height,
                    rgb: accumulator.to_rgb8(),
                    passes: accumulator.passes(),
                };
                if sender.send(frame).is_err() {
                    return; // The handle was dropped: render cancelled.
                }
                on_frame();
            }
        });

        Self { frames }
    }

    /// The newest frame since the last call, if any. Older frames that the UI
    /// didn't get to are skipped.
    pub fn latest_frame(&self) -> Option<Frame> {
        self.frames.try_iter().last()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn accumulator_averages_passes() {
        let mut acc = Accumulator::new(2);
        acc.add(&[Color::ZERO, Color::ONE]);
        acc.add(&[Color::new(0.5, 0.5, 0.5), Color::ONE]);
        assert_eq!(acc.passes(), 2);
        // Average 0.25 → gamma 0.5 → 128; average 1.0 → 255.
        assert_eq!(acc.to_rgb8(), vec![128, 128, 128, 255, 255, 255]);
    }

    #[test]
    fn empty_accumulator_is_black() {
        assert_eq!(Accumulator::new(1).to_rgb8(), vec![0, 0, 0]);
    }

    #[test]
    fn render_handle_delivers_every_pass() {
        let job = Job {
            scene: SceneChoice::Materials,
            camera: CameraConfig {
                image_width: 8,
                ..SceneChoice::Materials.build().camera
            },
            settings: RenderSettings {
                samples_per_pixel: 3,
                max_depth: 4,
            },
        };
        let handle = RenderHandle::start(job, || {});

        let mut passes = Vec::new();
        while passes.len() < 3 {
            let frame = handle
                .frames
                .recv_timeout(Duration::from_secs(10))
                .expect("render stalled");
            assert_eq!(frame.rgb.len(), (frame.width * frame.height * 3) as usize);
            passes.push(frame.passes);
        }
        assert_eq!(passes, [1, 2, 3]);
    }
}
