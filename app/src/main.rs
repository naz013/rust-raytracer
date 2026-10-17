mod app;
mod renderer;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title("Ray Tracer")
            .with_inner_size([1100.0, 700.0])
            .with_min_inner_size([640.0, 420.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Ray Tracer",
        options,
        Box::new(|cc| Ok(Box::new(app::RaytracerApp::new(cc)))),
    )
}
