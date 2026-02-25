// Game logic and struct implementation

pub const BOARD_SIZE: usize = 15;
pub const DIRECTIONS: [(isize, isize); 4] = [
    (1, 0), (0, 1), (1, 1), (1, -1), 
];


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
}

impl Default for Board {
    fn default() -> Self {
        Self { cells: [[Cell::Empty; BOARD_SIZE]; BOARD_SIZE], }
    }
}

impl Board {
    pub fn place(&mut self, x: usize, y: usize, player: Player) {
        self.cells[y][x] = player.to_cell();
    }

    pub fn is_empty(&self, x: usize, y: usize) -> bool {
        self.cells[y][x] == Cell::Empty
    }

}

pub enum GameState {
    Menu,
    Playing { mode: GameMode },
    GameOver { winner: Option<Player> },
}

pub enum GameMode {
    Solo,
    TwoPlayer,
}

pub struct GameSession {
    pub board: [[Option<Player>; 15]; 15],
    pub current_player: Player,
    pub state: GameState,
    pub move_history: Vec<(usize, usize)>,
    pub game_mode: GameMode,
}

impl Default for GameSession {
    fn default() -> Self {
        Self {
            board: [[None; 15]; 15],
            current_player: Player::Black,
            state: GameState::Menu,
            move_history: Vec::new(),
            game_mode: GameMode::TwoPlayer,
        }
    }
}