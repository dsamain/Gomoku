// Game logic and struct implementation

pub const BOARD_SIZE: usize = 19;
pub const DIRECTIONS: [(isize, isize); 4] = [
    (1, 0), (0, 1), (1, 1), (1, -1),
];
pub const WIN_COUNT: usize = 5;
pub const CAPTURE_WIN: u32 = 10;

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

pub fn in_bounds(x: isize, y: isize) -> bool {
    x >= 0 && x < BOARD_SIZE as isize && y >= 0 && y < BOARD_SIZE as isize
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

    /// Check for captures after placing at (x, y) for player.
    /// Removes captured pairs and returns the number of stones captured.
    pub fn check_captures(&mut self, x: usize, y: usize, player: Player) -> u32 {
        let own = player.to_cell();
        let opponent = player.next().to_cell();
        let mut captured = 0;

        for &(dx, dy) in DIRECTIONS.iter() {
            for dir in [1isize, -1isize] {
                let p1 = (x as isize + dx * dir,     y as isize + dy * dir);
                let p2 = (x as isize + dx * dir * 2, y as isize + dy * dir * 2);
                let p3 = (x as isize + dx * dir * 3, y as isize + dy * dir * 3);

                if !in_bounds(p1.0, p1.1) || !in_bounds(p2.0, p2.1) || !in_bounds(p3.0, p3.1) {
                    continue;
                }

                if self.cells[p1.1 as usize][p1.0 as usize] == opponent
                    && self.cells[p2.1 as usize][p2.0 as usize] == opponent
                    && self.cells[p3.1 as usize][p3.0 as usize] == own
                {
                    self.cells[p1.1 as usize][p1.0 as usize] = Cell::Empty;
                    self.cells[p2.1 as usize][p2.0 as usize] = Cell::Empty;
                    captured += 2;
                }
            }
        }

        captured
    }

    // -------------------------------------------------------------------------
    // Double-three detection
    // -------------------------------------------------------------------------

    /// Returns true if placing at (x,y) for player creates a double-three.
    pub fn is_double_three(&self, x: usize, y: usize, player: Player) -> bool {
        let mut temp = *self;
        temp.cells[y][x] = player.to_cell();

        let free_three_count = DIRECTIONS.iter()
            .filter(|&&(dx, dy)| temp.is_free_three(x, y, dx, dy, player))
            .count();

        // temp to debug all free three coordinates
        for &(dx, dy) in DIRECTIONS.iter() {
            if temp.is_free_three(x, y, dx, dy, player) {
                dbg!("Free three at:", (x, y), "direction:", (dx, dy));
            }
        }
        dbg!("double three count for:", (x, y), "is:", free_three_count);

        free_three_count >= 2
    }

    /// Returns true if there is a free-three through (x,y) in direction (dx,dy).
    /// A free-three is exactly 3 aligned stones that can be extended to an open four (.XXXX.)
    fn is_free_three(&self, x: usize, y: usize, dx: isize, dy: isize, player: Player) -> bool {
        let cell = player.to_cell();
        for start in -3isize..=0 {  // was -4..=0
            let positions: Vec<(isize, isize)> = (0..4)  // was 0..5
                .map(|i| (x as isize + dx * (start + i), y as isize + dy * (start + i)))
                .collect();
            if positions.iter().any(|&(px, py)| !in_bounds(px, py)) {
                continue;
            }
            if !positions.iter().any(|&(px, py)| px == x as isize && py == y as isize) {
                continue;
            }
            let cells: Vec<Cell> = positions.iter()
                .map(|&(px, py)| self.cells[py as usize][px as usize])
                .collect();
            let our_count = cells.iter().filter(|&&c| c == cell).count();
            let opponent_count = cells.iter().filter(|&&c| c != cell && c != Cell::Empty).count();
            if our_count != 3 || opponent_count != 0 {  // still 3 stones, 1 empty in window
                continue;
            }
            // ends just outside the 4-cell window
            let before = (positions[0].0 - dx, positions[0].1 - dy);
            let after  = (positions[3].0 + dx, positions[3].1 + dy);  // was positions[4]
            let before_open = !in_bounds(before.0, before.1)
                || self.cells[before.1 as usize][before.0 as usize] == Cell::Empty;
            let after_open = !in_bounds(after.0, after.1)
                || self.cells[after.1 as usize][after.0 as usize] == Cell::Empty;
            if before_open && after_open {
                return true;
            }
        }
        false
    }

    /// Count consecutive stones of `cell` through (x,y) in direction (dx,dy).
    /// Returns (count, is_left_open, is_right_open).
    fn count_sequence(&self, x: usize, y: usize, dx: isize, dy: isize, cell: Cell)
        -> (usize, bool, bool)
    {
        let (left_end, right_end) = self.sequence_endpoints(x, y, dx, dy, cell);

        let mut count = 0;
        let mut cx = left_end.0;
        let mut cy = left_end.1;
        loop {
            if self.cells[cy as usize][cx as usize] == cell {
                count += 1;
            }
            if (cx, cy) == right_end { break; }
            cx += dx;
            cy += dy;
        }

        let left_neighbor = (left_end.0 - dx, left_end.1 - dy);
        let right_neighbor = (right_end.0 + dx, right_end.1 + dy);

        let left_open = in_bounds(left_neighbor.0, left_neighbor.1)
            && self.cells[left_neighbor.1 as usize][left_neighbor.0 as usize] == Cell::Empty;
        let right_open = in_bounds(right_neighbor.0, right_neighbor.1)
            && self.cells[right_neighbor.1 as usize][right_neighbor.0 as usize] == Cell::Empty;

        (count, left_open, right_open)
    }

    /// Find the leftmost and rightmost positions of the sequence through (x,y).
    fn sequence_endpoints(&self, x: usize, y: usize, dx: isize, dy: isize, cell: Cell)
        -> ((isize, isize), (isize, isize))
    {
        let mut left = (x as isize, y as isize);
        let mut right = (x as isize, y as isize);

        let mut nx = x as isize - dx;
        let mut ny = y as isize - dy;
        while in_bounds(nx, ny) && self.cells[ny as usize][nx as usize] == cell {
            left = (nx, ny);
            nx -= dx;
            ny -= dy;
        }

        let mut nx = x as isize + dx;
        let mut ny = y as isize + dy;
        while in_bounds(nx, ny) && self.cells[ny as usize][nx as usize] == cell {
            right = (nx, ny);
            nx += dx;
            ny += dy;
        }

        (left, right)
    }

    fn check_winner_from(&self, x: usize, y: usize) -> Option<Player> {
        let cell = self.cells[y][x];
        if cell == Cell::Empty { return None; }

        let player = match cell {
            Cell::Black => Player::Black,
            Cell::White => Player::White,
            _ => unreachable!(),
        };

        for &(dx, dy) in DIRECTIONS.iter() {
            let mut count = 1;
            for dir in [1isize, -1isize] {
                let mut nx = x as isize + dx * dir;
                let mut ny = y as isize + dy * dir;
                while in_bounds(nx, ny) {
                    if self.cells[ny as usize][nx as usize] != cell || count >= WIN_COUNT {
                        break;
                    }
                    count += 1;
                    nx += dx * dir;
                    ny += dy * dir;
                }
            }
            if count >= WIN_COUNT {
                return Some(player);
            }
        }
        None
    }
}

// -------------------------------------------------------------------------

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

use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct EmotionState {
    pub emotion: AiEmotion,
    pub until: Option<Instant>,
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
                self.emotion = self.def;
                self.until = None;
            }
        }
    }
}

// -------------------------------------------------------------------------

#[derive(Debug)]
pub struct GameSession {
    pub board: Board,
    pub current_player: Player,
    pub state: GameState,
    pub move_history: Vec<(usize, usize)>,
    pub game_mode: GameMode,
    pub ai_turn: Player,
    pub ai_emotion: EmotionState,
    pub black_captures: u32,
    pub white_captures: u32,
    /// Cached forbidden moves (double-three) for the current player.
    /// Recomputed once after each move — free to read in UI without extra cost.
    pub forbidden_moves: Vec<(usize, usize)>,
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
            black_captures: 0,
            white_captures: 0,
            forbidden_moves: Vec::new(), // empty at start, no stones placed yet
        }
    }
}

impl GameSession {
    pub fn captures_for(&self, player: Player) -> u32 {
        match player {
            Player::Black => self.black_captures,
            Player::White => self.white_captures,
        }
    }

    pub fn is_forbidden(&self, x: usize, y: usize) -> bool {
        self.forbidden_moves.contains(&(x, y))
    }

    /// Recompute forbidden moves for the current player.
    /// Called once after each move — O(BOARD_SIZE^2 * 4 directions).
    pub fn recompute_forbidden(&mut self) {
        self.forbidden_moves.clear();
        let player = self.current_player;
        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                if self.board.is_empty(x, y) && self.board.is_double_three(x, y, player) {
                    self.forbidden_moves.push((x, y));
                }
            }
        }
    }

    /// Apply the move — place stone, captures, win check. Does NOT switch turn.
    pub fn apply_move(&mut self, x: usize, y: usize) -> bool { // returns false if invalid
        if self.state != GameState::Playing { return false; }
        if !self.board.is_empty(x, y) { return false; }
        if self.is_forbidden(x, y) { return false; }

        let player = self.current_player;
        self.board.place(x, y, player);
        self.move_history.push((x, y));

        let captured = self.board.check_captures(x, y, player);
        match player {
            Player::Black => self.black_captures += captured,
            Player::White => self.white_captures += captured,
        }

        if self.captures_for(player) >= CAPTURE_WIN {
            self.state = GameState::GameOver { winner: Some(player) };
        } else if let Some(winner) = self.board.winner() {
            self.state = GameState::GameOver { winner: Some(winner) };
        }

        true
    }

    /// Advance to the next turn — switch player and recompute forbidden.
    pub fn advance_turn(&mut self) {
        self.current_player = self.current_player.next();
        self.recompute_forbidden();
    }

    /// Convenience: apply move and advance turn (normal gameplay)
    pub fn play_at(&mut self, x: usize, y: usize) {
        if self.apply_move(x, y) && self.state == GameState::Playing {
            self.advance_turn();
        }
    }
}