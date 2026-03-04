
use eframe::egui::{CentralPanel, ColorImage};
use std::sync::mpsc;
use std::collections::HashMap;

use crate::game::{GameSession, GameState, AiEmotion, GameMode};
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
    pub ai_rx: Option<mpsc::Receiver<Result<((usize, usize), crate::ai::AiStats), String>>>,
    pub ai_last_stats: Option<crate::ai::AiStats>,
    pub ai_thinking: bool,
    pub ai_think_start: Option<std::time::Instant>,
    pub ai_last_think_duration: Option<f32>,

    pub ai_textures: HashMap<AiEmotion, egui::TextureHandle>,

    // Settings
    pub game_mode: GameMode,
    pub ai_depth: u32,
    pub debug_no_turn_pass: bool,
    pub debug_mode: bool,
}

impl Default for GomokuApp {
    fn default() -> Self {
        Self {
            screen: Screen::Menu,
            session: None,
            ai_rx: None,
            ai_last_stats: None,
            ai_thinking: false,
            ai_think_start: None,
            ai_last_think_duration: None,

            ai_textures: HashMap::new(),

            game_mode: GameMode::Solo,
            ai_depth: 4,
            debug_mode: false,
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
                            Ok(((x, y), stats)) => {
                                if let Some(session) = &mut self.session {
                                    session.play_at(x, y);
                                }
                                self.ai_last_stats = Some(stats);
                            }
                            Err(e) => {
                                dbg!("AI error:", e);
                                self.start_game();
                            }
                        }
                        self.ai_thinking = false;
                        self.ai_last_think_duration = self.ai_think_start.map(|start| start.elapsed().as_secs_f32());
                        self.ai_think_start = None;
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
        dbg!("Starting new game with mode:", &self.game_mode, "and AI depth:", self.ai_depth);
        let mut session = GameSession::default();
        session.state = GameState::Playing;
        session.game_mode = self.game_mode.clone();
        self.session = Some(session);
        self.screen = Screen::Game;
        self.ai_last_stats = None;

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
