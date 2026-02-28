use eframe::egui;
use egui::{Align, CentralPanel, Layout, RichText, Vec2};
use crate::app::{GomokuApp, Screen};

pub fn show(ctx: &egui::Context, app: &mut GomokuApp) {
    CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(60.0);
            ui.label(RichText::new("Settings").size(36.0).strong());
            ui.add_space(30.0);

            // --- Game ---
            ui.label(RichText::new("🎮  Game").size(18.0).strong());
            ui.add_space(8.0);
            ui.checkbox(&mut app.two_player_mode, "Two player mode");

            ui.add_space(20.0);

            // --- Debug ---
            ui.label(RichText::new("🛠  Debug").size(18.0).strong());
            ui.add_space(8.0);
            ui.checkbox(&mut app.debug_mode, "Debug mode");

            ui.add_space(30.0);
            ui.separator();
            ui.add_space(20.0);

            let button_size = Vec2::new(200.0, 45.0);
            if ui.add_sized(button_size, egui::Button::new(
                RichText::new("← Back").size(18.0)
            )).clicked() {
                app.screen = Screen::Menu;
            }
        });
    });
}