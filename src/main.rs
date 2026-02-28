use eframe::egui;
use app::GomokuApp;

mod app;
mod game;
mod menu;
mod board;
mod ai;
mod settings;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 700.0])
            .with_title("Gomoku"),
        ..Default::default()
    };

    eframe::run_native(
        "Gomoku",
        options,
        Box::new(|cc| Box::new(GomokuApp::new(cc))))
}
