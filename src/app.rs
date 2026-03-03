
use eframe::egui::{CentralPanel, ColorImage};
use std::sync::mpsc;
use std::collections::HashMap;

use crate::game::{GameSession, GameState, AiEmotion};
use crate::{egui, settings, menu, board};


#[derive(Debug, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Game,
    Settings,
}

pub struct GomokuApp {
    pub screen: Screen,
    pub session: Option<GameSession>,
    pub ai_rx: Option<mpsc::Receiver<Result<(usize, usize), String>>>,
    pub ai_thinking: bool,
    pub ai_textures: HashMap<AiEmotion, egui::TextureHandle>,

    // Settings
    pub debug_mode: bool,
    pub two_player_mode: bool,
    pub debug_no_turn_pass: bool,
}

impl Default for GomokuApp {
    fn default() -> Self {
        Self {
            screen: Screen::Menu,
            session: None,
            ai_rx: None,
            ai_thinking: false,
            ai_textures: HashMap::new(),

            two_player_mode: false,
            debug_mode: true,
            debug_no_turn_pass: false,
        }
    }
}

impl eframe::App for GomokuApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        CentralPanel::default().show(ctx, |ui| {

            if self.ai_thinking {
                if let Some(rx) = &self.ai_rx {
                    if let Ok(inner) = rx.try_recv() {
                        match inner {
                            Ok((x, y)) => {
                                if let Some(session) = &mut self.session {
                                    session.play_at(x, y);
                                }
                            }
                            Err(e) => {
                                dbg!("AI error:", e);
                                self.start_game();
                            }
                        }
                        self.ai_thinking = false;
                        self.ai_rx = None;
                    }
                }
                ctx.request_repaint(); // request a repaint manually as it's called only when events happend by default
            }

            if let Some(session) = &mut self.session {
                session.ai_emotion.tick();

                let current_emotion = session.ai_emotion.emotion;
                if self.ai_thinking {
                    session.ai_emotion.set_default(AiEmotion::Thinking);
                } else  {
                    session.ai_emotion.set_default(AiEmotion::Neutral);
                }

                if current_emotion != session.ai_emotion.emotion {
                    ctx.request_repaint();
                }
            }

            match self.screen {
                Screen::Menu => menu::show(ctx, self),
                Screen::Game => board::show(ctx, self),
                Screen::Settings => settings::show(ctx, self),
                // _ => unreachable!(),
            }
        });
        // match self.current_screen {
        //     Screen::Menu => self.show_menu(ctx),
        //     Screen::Game => self.show_game(ctx),
        //     Screen::Settings => self.show_settings(ctx),
        // }
    }
}

fn load_texture(ctx: &egui::Context, name: &str, bytes: &[u8]) -> egui::TextureHandle {
    let image = image::load_from_memory(bytes).unwrap();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = image.to_rgba8();
    ctx.load_texture(
        name,
        egui::ColorImage::from_rgba_unmultiplied(size, &pixels),
        Default::default()
    )
}

impl GomokuApp {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let ctx = &cc.egui_ctx;
        let mut textures = HashMap::new();

        textures.insert(AiEmotion::Neutral,  load_texture(ctx, "neutral",  include_bytes!("../assets/neutral.png")));
        textures.insert(AiEmotion::Thinking, load_texture(ctx, "thinking", include_bytes!("../assets/thinking.png")));
        textures.insert(AiEmotion::Happy,    load_texture(ctx, "happy",    include_bytes!("../assets/happy.png")));
        textures.insert(AiEmotion::Angry,    load_texture(ctx, "angry",    include_bytes!("../assets/angry.png")));
        textures.insert(AiEmotion::Surprised,load_texture(ctx, "surprised",include_bytes!("../assets/surprised.png")));

        Self {
            ai_textures: textures,
            ..Default::default()
        }
    }

    // when start game, session is built from default, then parameter from app are applied
    pub fn start_game(&mut self) {
        self.session = Some(GameSession::default());
        // self.session.as_mut().unwrap().state = GameState::Playing;
        if let Some(session) = &mut self.session {
            session.state = GameState::Playing;
            if self.two_player_mode {
                session.game_mode = crate::game::GameMode::TwoPlayer;
            } else {
                session.game_mode = crate::game::GameMode::Solo;
            }
        }
        self.screen = Screen::Game;
    }

    pub fn back_to_menu(&mut self) {
        self.session = None;
        self.screen = Screen::Menu;
    }

    pub fn is_game_over(&self) -> bool {
        self.session
            .as_ref()
            .map(|s| matches!(s.state, GameState::GameOver { .. }))
            .unwrap_or(false)
    }
}
