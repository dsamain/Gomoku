use crate::egui::{Align, CentralPanel, Layout, RichText, Vec2};
use eframe::egui;
use crate::app::{GomokuApp, Screen};

pub fn show(ctx: &egui::Context, app: &mut GomokuApp) {
    CentralPanel::default().show(ctx, |ui| {
        // center everything vertically and horizontally
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(120.0);

            ui.label(RichText::new("GOMOKU").size(64.0).strong());
            ui.label(RichText::new("Five in a row").size(18.0).weak());

            ui.add_space(60.0);

            let button_size = Vec2::new(200.0, 45.0);

            if ui.add_sized(button_size, egui::Button::new(
                RichText::new("▶  New Game").size(18.0)
            )).clicked() {
                app.start_game();
            }

            ui.add_space(10.0);

            if ui.add_sized(button_size, egui::Button::new(
                RichText::new("⚙  Settings").size(18.0)
            )).clicked() {
                app.screen = Screen::Settings;
            }

            ui.add_space(10.0);

            if ui.add_sized(button_size, egui::Button::new(
                RichText::new("✕  Quit").size(18.0)
            )).clicked() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
            }
        });
    });
}