mod app;
mod generator_engine;
mod pages;

use app::ParserApp;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "LR(0) Parser GUI",
        options,
        Box::new(|_cc| Ok(Box::new(ParserApp::default()))),
    )
}
