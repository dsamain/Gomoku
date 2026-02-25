use eframe::egui;

mod app;
mod game;


fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        initial_window_size: Some(egui::vec2(800.0, 600.0)),
        ..Default::default()
    };
    eframe::run_native(
        "Gomoku",
        options,
        Box::new(|_cc| Box::new(game::GameApp::new())),
    )   
}