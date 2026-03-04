use eframe::egui;
use egui::{Align, CentralPanel, Layout, RichText, Vec2};
use crate::app::{GomokuApp, Screen};
use crate::game::GameMode;

pub fn show(ctx: &egui::Context, app: &mut GomokuApp) {
    CentralPanel::default().show(ctx, |ui| {
        ui.with_layout(Layout::top_down(Align::Center), |ui| {
            ui.add_space(60.0);
            ui.label(RichText::new("Settings").size(36.0).strong());
            ui.add_space(30.0);

            // --- Game Mode ---
            ui.label(RichText::new("🎮  Game Mode").size(18.0).strong());
            ui.add_space(12.0);

            let button_size = Vec2::new(200.0, 38.0);

            let solo_btn = egui::Button::new(RichText::new("👤  Solo").size(16.0))
                .selected(app.game_mode == GameMode::Solo);
            if ui.add_sized(button_size, solo_btn).clicked() {
                app.game_mode = GameMode::Solo;
            }

            ui.add_space(6.0);

            let two_btn = egui::Button::new(RichText::new("👥  Two Players").size(16.0))
                .selected(app.game_mode == GameMode::TwoPlayer);
            if ui.add_sized(button_size, two_btn).clicked() {
                app.game_mode = GameMode::TwoPlayer;
            }

            ui.add_space(6.0);

            let ai_btn = egui::Button::new(RichText::new("🤖  AI vs AI").size(16.0))
                .selected(app.game_mode == GameMode::AiVsAi);
            if ui.add_sized(button_size, ai_btn).clicked() {
                app.game_mode = GameMode::AiVsAi;
            }

            ui.add_space(24.0);

            // --- AI Depth — hidden in two player mode ---
            if app.game_mode != GameMode::TwoPlayer {
                ui.label(RichText::new("🧠  AI Depth").size(18.0).strong());
                ui.add_space(8.0);
                ui.label(RichText::new(format!("Depth: {}", app.ai_depth))
                    .size(14.0)
                    .color(egui::Color32::from_rgb(150, 150, 150)));
                ui.add_space(4.0);
                ui.add_sized(
                    Vec2::new(100.0, 20.0),
                    egui::Slider::new(&mut app.ai_depth, 1..=8).show_value(false)
                );
                ui.add_space(4.0);
                let depth_label = match app.ai_depth {
                    1..=2 => "Easy",
                    3..=4 => "Medium",
                    5..=6 => "Hard",
                    _     => "Very Hard",
                };
                ui.label(RichText::new(depth_label)
                    .size(13.0)
                    .color(egui::Color32::from_rgb(100, 140, 200)));
                ui.add_space(20.0);
            }

            // --- Debug ---
            ui.label(RichText::new("🛠  Debug").size(18.0).strong());
            ui.add_space(8.0);
            ui.checkbox(&mut app.debug_mode, "Debug mode");

            ui.add_space(30.0);
            ui.separator();
            ui.add_space(20.0);

            if ui.add_sized(Vec2::new(200.0, 45.0), egui::Button::new(
                RichText::new("← Back").size(18.0)
            )).clicked() {
                app.screen = Screen::Menu;
            }
        });
    });
}