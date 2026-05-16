mod app;
mod generator_engine;
mod pages;

use app::ParserApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    eframe::run_native(
        "LR(0) Parser GUI",
        options,
        Box::new(|_cc| Ok(Box::new(ParserApp::default()))),
    )
}
