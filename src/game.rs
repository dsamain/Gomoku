// Game logic and struct implementation

pub const BOARD_SIZE: usize = 19;
pub const DIRECTIONS: [(isize, isize); 4] = [
    (1, 0), (0, 1), (1, 1), (1, -1), 
];
pub const WIN_COUNT: usize = 5;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Black,
    White,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Player {
    Black,
    White,
}

impl Player {
    pub fn to_cell(self) -> Cell {
        match self {
            Player::Black => Cell::Black,
            Player::White => Cell::White,
        }
    }

    pub fn next(self) -> Player {
        match self {
            Player::Black => Player::White,
            Player::White => Player::Black,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Player::Black => "Black",
            Player::White => "White",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Board {
    pub cells: [[Cell; BOARD_SIZE]; BOARD_SIZE],
    winner: Option<Player>,
}

impl Default for Board {
    fn default() -> Self {
        Self {
            cells: [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE],
            winner: None,
        }
    }
}

impl Board {
    pub fn place(&mut self, x: usize, y: usize, player: Player) {
        self.cells[y][x] = player.to_cell();
        if let Some(winner) = self.check_winner_from(x, y) {
            self.winner = Some(winner);
        }
    }

    pub fn is_empty(&self, x: usize, y: usize) -> bool {
        self.cells[y][x] == Cell::Empty
    }

    pub fn winner(&self) -> Option<Player> {
        self.winner
    }

    fn check_winner_from(&self, x: usize, y: usize) -> Option<Player> {
        let cell = self.cells[y][x];
        if cell == Cell::Empty {
            return None;
        }
        let player = match cell {
            Cell::Black => Player::Black,
            Cell::White => Player::White,
            _ => unreachable!(),
        };

        for &(dx, dy) in DIRECTIONS.iter() {
            let mut count = 1;
            for dir in [1, -1].iter() {
                let mut nx = x as isize + dx * *dir;
                let mut ny = y as isize + dy * *dir;
                while nx >= 0 && nx < BOARD_SIZE as isize && ny >= 0 && ny < BOARD_SIZE as isize {
                    if self.cells[ny as usize][nx as usize] != cell || count >= WIN_COUNT {
                        break;
                    }
                    count += 1;
                    nx += dx * *dir;
                    ny += dy * *dir;
                }
            }
            if count >= WIN_COUNT {
                return Some(player);
            }
        }
        None
    }

}

#[derive(Clone, Debug, PartialEq)]
pub enum GameState {
    Menu,
    Playing,
    GameOver { winner: Option<Player> },
}

#[derive(Clone, Debug, PartialEq)]
pub enum GameMode {
    Solo,
    TwoPlayer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AiEmotion {
    Neutral,
    Thinking,
    Happy,
    Angry,
    Surprised,
}
use std::time::{Instant, Duration};

#[derive(Debug)]
pub struct EmotionState {
    pub emotion: AiEmotion,
    pub until: Option<Instant>, // None = permanent, Some = expires at this time
    def: AiEmotion,
    permanent: bool,
}

impl Default for EmotionState {
    fn default() -> Self {
        Self {
            emotion: AiEmotion::Neutral,
            until: None,
            def: AiEmotion::Neutral,
            permanent: false,
        }
    }
}

impl EmotionState {
    pub fn set_permanent(&mut self, emotion: AiEmotion) {
        self.emotion = emotion;
        self.until = None;
        self.permanent = true;
    }

    pub fn set_default(&mut self, emotion: AiEmotion) {
        self.def = emotion;
        if self.until.is_none() && !self.permanent {
            self.emotion = emotion;
        }
    }

    pub fn set_timed(&mut self, emotion: AiEmotion, duration: Duration) {
        self.permanent = false;
        self.emotion = emotion;
        self.until = Some(Instant::now() + duration);
    }

    pub fn clear_timed(&mut self) {
        self.emotion = self.def;
        self.until = None;
    }

    pub fn tick(&mut self) {
        if let Some(until) = self.until {
            if Instant::now() >= until {
                dbg!("Emotion expired, reverting to default:", self.def);
                self.emotion = self.def;
                self.until = None;
            }
        }
    }
}

#[derive(Debug)]
pub struct GameSession {
    pub board: Board,
    pub current_player: Player,
    pub state: GameState,
    pub move_history: Vec<(usize, usize)>,
    pub game_mode: GameMode,
    // pub ai: Option<crate::ai::AI>,
    pub ai_turn: Player,
    pub ai_emotion: EmotionState,
    pub black_captures: u32,
    pub white_captures: u32,
}

impl Default for GameSession {
    fn default() -> Self {
        Self {
            board: Board::default(),
            current_player: Player::Black,
            state: GameState::Menu,
            move_history: Vec::new(),
            game_mode: GameMode::Solo,
            ai_turn: Player::White,
            ai_emotion: EmotionState::default(),
        }
    }
}

impl GameSession {
    pub fn play_at(&mut self, x: usize, y: usize) {
        if self.state != GameState::Playing { return; }
        if !self.board.is_empty(x, y) { return; }

        self.board.place(x, y, self.current_player);
        self.move_history.push((x, y));

        if let Some(winner) = self.board.winner() {
            self.state = GameState::GameOver { winner: Some(winner) };
        } else {
            self.current_player = self.current_player.next();
        } 
    }
}
