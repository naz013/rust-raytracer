use std::time::{Duration, Instant};

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use image::RgbImage;
use tracer::{CameraConfig, RenderSettings, Vec3};

use crate::renderer::{Frame, Job, RenderHandle, SceneChoice};

/// Default image width in the app: smaller than the CLI's so passes are quick
/// and the picture refines visibly.
const DEFAULT_WIDTH: u32 = 600;

/// The render in progress, or the last one that finished.
struct ActiveRender {
    /// Dropping this (by replacing the whole `ActiveRender`) cancels the render.
    handle: RenderHandle,
    job: Job,
    started: Instant,
    elapsed: Duration,
    passes: u32,
}

pub struct RaytracerApp {
    // What the controls are currently set to.
    scene: SceneChoice,
    camera: CameraConfig,
    settings: RenderSettings,

    render: Option<ActiveRender>,
    /// The latest frame, kept for display and saving.
    frame: Option<Frame>,
    texture: Option<TextureHandle>,

    save_path: String,
    status: String,
}

impl RaytracerApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let scene = SceneChoice::Cover;
        let mut app = Self {
            scene,
            camera: CameraConfig {
                image_width: DEFAULT_WIDTH,
                ..scene.build().camera
            },
            settings: RenderSettings::default(),
            render: None,
            frame: None,
            texture: None,
            save_path: "render.png".to_string(),
            status: String::new(),
        };
        app.start_render(&cc.egui_ctx);
        app
    }

    fn current_job(&self) -> Job {
        Job {
            scene: self.scene,
            camera: self.camera.clone(),
            settings: self.settings.clone(),
        }
    }

    fn start_render(&mut self, ctx: &egui::Context) {
        let job = self.current_job();
        // egui::Context is a cheap, thread-safe handle (internally an Arc),
        // so the worker can keep a clone to request repaints.
        let ctx = ctx.clone();
        let handle = RenderHandle::start(job.clone(), move || ctx.request_repaint());

        // Assigning drops the previous ActiveRender, which cancels it.
        self.render = Some(ActiveRender {
            handle,
            job,
            started: Instant::now(),
            elapsed: Duration::ZERO,
            passes: 0,
        });
        self.status.clear();
    }

    /// Pulls the newest frame from the worker into the texture.
    fn poll_render(&mut self, ctx: &egui::Context) {
        let Some(render) = &mut self.render else {
            return;
        };
        let Some(frame) = render.handle.latest_frame() else {
            return;
        };
        render.passes = frame.passes;
        render.elapsed = render.started.elapsed();

        let size = [frame.width as usize, frame.height as usize];
        let image = ColorImage::from_rgb(size, &frame.rgb);
        match &mut self.texture {
            Some(texture) => texture.set(image, TextureOptions::LINEAR),
            None => self.texture = Some(ctx.load_texture("render", image, TextureOptions::LINEAR)),
        }
        self.frame = Some(frame);
    }

    fn save_png(&mut self) {
        let Some(frame) = &self.frame else {
            self.status = "Nothing to save yet".to_string();
            return;
        };
        let result = RgbImage::from_raw(frame.width, frame.height, frame.rgb.clone())
            .expect("frame buffer matches its size")
            .save(&self.save_path);
        self.status = match result {
            Ok(()) => format!("Saved {} ({} samples)", self.save_path, frame.passes),
            Err(err) => format!("Save failed: {err}"),
        };
    }

    fn controls(&mut self, ui: &mut egui::Ui) {
        ui.heading("Ray Tracer");
        ui.add_space(8.0);

        let previous_scene = self.scene;
        egui::ComboBox::from_label("Scene")
            .selected_text(self.scene.label())
            .show_ui(ui, |ui| {
                for choice in SceneChoice::ALL {
                    ui.selectable_value(&mut self.scene, choice, choice.label());
                }
            });
        if self.scene != previous_scene {
            // Switch to the new scene's camera, but keep the chosen resolution.
            self.camera = CameraConfig {
                image_width: self.camera.image_width,
                ..self.scene.build().camera
            };
            self.start_render(ui.ctx());
        }

        section(ui, "Quality");
        ui.add(egui::Slider::new(&mut self.camera.image_width, 100..=1600).text("width (px)"));
        ui.add(
            egui::Slider::new(&mut self.settings.samples_per_pixel, 1..=1000)
                .logarithmic(true)
                .text("samples / pixel"),
        );
        ui.add(egui::Slider::new(&mut self.settings.max_depth, 1..=100).text("max bounces"));

        section(ui, "Camera");
        ui.label("Position");
        vec3_editor(ui, &mut self.camera.look_from);
        ui.label("Look at");
        vec3_editor(ui, &mut self.camera.look_at);
        ui.add(
            egui::Slider::new(&mut self.camera.vfov, 5.0..=120.0)
                .suffix("°")
                .text("field of view"),
        );
        ui.add(
            egui::Slider::new(&mut self.camera.defocus_angle, 0.0..=10.0)
                .suffix("°")
                .text("aperture"),
        );
        ui.add(
            egui::Slider::new(&mut self.camera.focus_dist, 0.1..=50.0)
                .logarithmic(true)
                .text("focus distance"),
        );
        if ui.button("Focus on target").clicked() {
            self.camera.focus_dist = (self.camera.look_from - self.camera.look_at).length();
        }

        section(ui, "Render");
        let job = self.current_job();
        let changed = self.render.as_ref().is_none_or(|r| r.job != job);
        if ui.button("Re-render").clicked() {
            self.start_render(ui.ctx());
        }
        if changed {
            ui.colored_label(
                ui.visuals().warn_fg_color,
                "Settings changed: re-render to apply",
            );
        }
        if let Some(render) = &self.render {
            let total = render.job.settings.samples_per_pixel.max(1);
            let progress = render.passes as f32 / total as f32;
            ui.add(egui::ProgressBar::new(progress).text(format!(
                "{} / {total} samples · {:.1} s",
                render.passes,
                render.elapsed.as_secs_f32()
            )));
        }

        section(ui, "Save");
        ui.horizontal(|ui| {
            ui.add(egui::TextEdit::singleline(&mut self.save_path).desired_width(140.0));
            if ui.button("Save PNG").clicked() {
                self.save_png();
            }
        });
        if !self.status.is_empty() {
            ui.label(&self.status);
        }
    }

    fn image_view(&self, ui: &mut egui::Ui) {
        match &self.texture {
            Some(texture) => {
                ui.centered_and_justified(|ui| {
                    ui.add(egui::Image::from_texture(texture).shrink_to_fit());
                });
            }
            None => {
                ui.centered_and_justified(|ui| ui.spinner());
            }
        }
    }
}

impl eframe::App for RaytracerApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_render(ui.ctx());

        egui::Panel::left("controls")
            .resizable(false)
            .default_size(280.0)
            .show(ui, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| self.controls(ui));
            });
        egui::CentralPanel::default().show(ui, |ui| self.image_view(ui));
    }
}

fn section(ui: &mut egui::Ui, title: &str) {
    ui.add_space(8.0);
    ui.separator();
    ui.strong(title);
}

fn vec3_editor(ui: &mut egui::Ui, v: &mut Vec3) {
    ui.horizontal(|ui| {
        for (label, value) in [("x ", &mut v.x), ("y ", &mut v.y), ("z ", &mut v.z)] {
            ui.add(egui::DragValue::new(value).speed(0.05).prefix(label));
        }
    });
}
