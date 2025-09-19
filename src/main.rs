use eframe::NativeOptions;

mod app;
mod generator_engine;
mod pages;

use app::ParserApp;

// GUIアプリケーションのエントリーポイント
fn main() -> eframe::Result<()> {
    let options = NativeOptions::default();
    eframe::run_native(
        "LR(0) Parser",
        options,
        Box::new(|_cc| Ok(Box::<ParserApp>::default())),
    )
}
