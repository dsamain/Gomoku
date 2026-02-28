use std::sync::mpsc;
use std::time::Duration;
use eframe::egui;
use egui::{CentralPanel, Color32, Pos2, Rect, Sense, SidePanel, Stroke, Vec2, RichText};
use rand::Rng;

use crate::app::GomokuApp;
use crate::game::{Cell, GameState, BOARD_SIZE, EmotionState, AiEmotion, GameMode};

const CELL_SIZE: f32 = 28.0;
const STONE_RADIUS: f32 = 11.0;
const BOARD_PADDING: f32 = 23.0;

pub fn show(ctx: &egui::Context, app: &mut GomokuApp) {
    // --- side panel: info + controls ---
    SidePanel::right("side_panel")
        .exact_width(160.0)
        .show(ctx, |ui| {
            ui.add_space(20.0);
            ui.label(RichText::new("Gomoku").size(20.0).strong());
            ui.separator();

            if let Some(session) = &app.session {
                ui.add_space(10.0);

                match &session.state {
                    GameState::Playing => {
                        ui.label("Current turn:");
                        let (color, name) = match session.current_player {
                            crate::game::Player::Black => (Color32::BLACK, "Black"),
                            crate::game::Player::White => (Color32::WHITE, "White"),
                        };
                        ui.horizontal(|ui| {
                            let (rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                            ui.painter().circle_filled(rect.center(), 7.0, color);
                            ui.painter().circle_stroke(rect.center(), 7.0, Stroke::new(1.0, Color32::GRAY));
                            ui.label(name);
                        });

                        ui.add_space(10.0);
                        ui.separator();
                        ui.add_space(10.0);

                        ui.label("Captures:");
                        ui.add_space(4.0);

                        // black captures
                        ui.horizontal(|ui| {
                            let (rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                            ui.painter().circle_filled(rect.center(), 7.0, Color32::BLACK);
                            ui.painter().circle_stroke(rect.center(), 7.0, Stroke::new(1.0, Color32::GRAY));
                            ui.label(format!("{} / 10", session.black_captures));
                        });

                        ui.add_space(4.0);

                        // white captures
                        ui.horizontal(|ui| {
                            let (rect, _) = ui.allocate_exact_size(Vec2::splat(16.0), Sense::hover());
                            ui.painter().circle_filled(rect.center(), 7.0, Color32::WHITE);
                            ui.painter().circle_stroke(rect.center(), 7.0, Stroke::new(1.0, Color32::GRAY));
                            ui.label(format!("{} / 10", session.white_captures));
                        });
                    }
                    GameState::GameOver { winner } => {
                        ui.label(RichText::new(format!("{} wins!", winner.unwrap().name()))
                            .size(16.0).strong().color(Color32::GOLD));
                    }
                    _ => {}
                }

                ui.add_space(10.0);
                // ui.label(format!("Moves: {}", session.move_count));
            }

            ui.add_space(20.0);
            ui.separator();

            if ui.button("↩  Menu").clicked() {
                app.back_to_menu();
            }

            if ui.button("↺  Restart").clicked() {
                app.start_game();
            }

            // --- debug section ---
            if app.debug_mode {
                ui.add_space(10.0);
                ui.separator();
                ui.label(RichText::new("🛠 Debug").size(13.0).strong());
                ui.add_space(4.0);
                ui.checkbox(&mut app.debug_no_turn_pass, "Don't pass turn");
            }

            // AI emotion display
            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                let emotion = app.session
                    .as_ref()
                    .map(|s| s.ai_emotion.emotion)  // only copy the AiEmotion, not the whole EmotionState
                    .unwrap_or(AiEmotion::Neutral);

                if let Some(texture) = app.ai_textures.get(&emotion) {
                    ui.add(egui::Image::new(texture).fit_to_exact_size(egui::vec2(120.0, 120.0)));
                }
            });
        });

    // --- main board area ---
    CentralPanel::default().show(ctx, |ui| {
        // Ai thread spawn
        if let Some(session) = &mut app.session {
            if session.game_mode == crate::game::GameMode::Solo
                && session.current_player == session.ai_turn
                && !app.ai_thinking
                && session.state == GameState::Playing
            {
                dbg!("Spawning AI thread...");
                app.ai_thinking = true;
                let board_clone = session.board.clone();
                let ai_player = session.ai_turn;
                let (tx, rx) = mpsc::channel();
                app.ai_rx = Some(rx);
                std::thread::spawn(move || {
                    let best = crate::ai::best_move(&board_clone, ai_player, 3);
                    tx.send(best).unwrap(); // unwrap is here so the program panic if send return an error
                });
            }
        }

        let board_px = (BOARD_SIZE - 1) as f32 * CELL_SIZE;
        let total = board_px + BOARD_PADDING * 2.0;

        // center the board in available space
        let available = ui.available_size();
        let offset = Vec2::new(
            (available.x - total).max(0.0) / 2.0,
            (available.y - total).max(0.0) / 2.0,
        );

        let origin = ui.min_rect().min + offset + Vec2::splat(BOARD_PADDING);

        // allocate the full board area for interaction
        let board_rect = Rect::from_min_size(
            ui.min_rect().min + offset,
            Vec2::splat(total),
        );
        let response = ui.allocate_rect(board_rect, Sense::click());

        let painter = ui.painter();

        // draw background
        painter.rect_filled(board_rect, 4.0, Color32::from_rgb(220, 179, 92));

        // draw grid lines
        for i in 0..BOARD_SIZE {
            let x = origin.x + i as f32 * CELL_SIZE;
            let y = origin.y + i as f32 * CELL_SIZE;

            painter.line_segment(
                [Pos2::new(origin.x, y), Pos2::new(origin.x + board_px, y)],
                Stroke::new(1.0, Color32::from_rgb(80, 50, 20)),
            );
            painter.line_segment(
                [Pos2::new(x, origin.y), Pos2::new(x, origin.y + board_px)],
                Stroke::new(1.0, Color32::from_rgb(80, 50, 20)),
            );
        }

        // draw star points (standard gomoku dots)
        for &(r, c) in &[(3,3),(3,11),(7,7),(11,3),(11,11)] {
            let pos = grid_to_pos(origin, r, c);
            painter.circle_filled(pos, 4.0, Color32::from_rgb(80, 50, 20));
        }

        // draw stones
        if let Some(session) = &app.session {
            for row in 0..BOARD_SIZE {
                for col in 0..BOARD_SIZE {
                    let cell = session.board.cells[row][col];
                    if cell == Cell::Empty { continue; }

                    let pos = grid_to_pos(origin, row, col);
                    let color = if cell == Cell::Black { Color32::BLACK } else { Color32::WHITE };
                    painter.circle_filled(pos, STONE_RADIUS, color);
                    painter.circle_stroke(pos, STONE_RADIUS, Stroke::new(1.0, Color32::DARK_GRAY));
                }
            }
            // draw forbidden move indicators for current player (free three)
            for &(fx, fy) in &session.forbidden_moves {
                let pos = grid_to_pos(origin, fy, fx);
                painter.circle_filled(pos, 5.0, Color32::from_rgba_premultiplied(255, 0, 0, 120));
            }
        }

        // handle click — find nearest intersection
        if response.clicked() {
            if let Some(pos) = response.interact_pointer_pos() {
                if let Some((row, col)) = pos_to_grid(origin, pos) {
                    if let Some(session) = &mut app.session {
                        if session.game_mode == GameMode::Solo && session.ai_turn == session.current_player {
                            dbg!("It's AI's turn, ignoring click");
                            return;
                        }
                        if app.debug_no_turn_pass {
                            session.apply_move(col, row);
                            session.recompute_forbidden();
                        } else {
                            session.play_at(col, row);
                        }
                        let mut rng = rand::thread_rng();
                        let roll: f64 = rng.gen_range(0.0..1.0);

                        match roll {
                            // r if r < 0.2 => session.ai_emotion.set_timed(AiEmotion::Angry, Duration::from_millis(400)),
                            r if r < 0.5 && r >= 0.2 => session.ai_emotion.set_timed(AiEmotion::Surprised, Duration::from_millis(500)),
                            _ => {}
                        }
                        dbg!("Clicked grid:", row, col);
                    }
                }
            }
        }

        // game over overlay
        if app.is_game_over() {
            if let Some(session) = &mut app.session {
                if let GameState::GameOver { winner } = &session.state {
                    let center = board_rect.center();
                    let rect = Rect::from_center_size(center, Vec2::new(220.0, 80.0));
                    painter.rect_filled(rect, 8.0, Color32::from_rgba_premultiplied(0, 0, 0, 180));
                    painter.text(
                        center,
                        egui::Align2::CENTER_CENTER,
                        format!("{} wins! 🎉", winner.unwrap().name()),
                        egui::FontId::proportional(28.0),
                        Color32::GOLD,
                    );
                    if winner.unwrap() == session.ai_turn {
                        session.ai_emotion.set_permanent(AiEmotion::Happy);
                    } else {
                        session.ai_emotion.set_permanent(AiEmotion::Angry);
                    }
                }
            }
        }
    });
}

fn grid_to_pos(origin: Pos2, row: usize, col: usize) -> Pos2 {
    Pos2::new(
        origin.x + col as f32 * CELL_SIZE,
        origin.y + row as f32 * CELL_SIZE,
    )
}

fn pos_to_grid(origin: Pos2, pos: Pos2) -> Option<(usize, usize)> {
    let col = ((pos.x - origin.x) / CELL_SIZE).round() as i32;
    let row = ((pos.y - origin.y) / CELL_SIZE).round() as i32;

    if row >= 0 && col >= 0 && row < BOARD_SIZE as i32 && col < BOARD_SIZE as i32 {
        Some((row as usize, col as usize))
    } else {
        None
    }
}