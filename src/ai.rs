use crate::game::{Board, Player, Cell, BOARD_SIZE, DIRECTIONS, CAPTURE_WIN, in_bounds};
use rand::Rng;

const INF: i32 = 10_000_000;
const WIN_SCORE: i32 = 1_000_000;
const MAX_CANDIDATES: usize = 20;

// -------------------------------------------------------------------------
// Search statistics — public so app.rs can store and board.rs can display
// -------------------------------------------------------------------------

#[derive(Default, Clone, Debug)]
pub struct AiStats {
    pub nodes: u64,
    pub pruned: u64,
    pub avg_branch: f64,
    pub time_ms: u128,
}

#[derive(Default)]
struct SearchStats {
    nodes: u64,
    pruned: u64,
    total_candidates: u64,
    candidate_calls: u64,
}

impl SearchStats {
    fn avg_candidates(&self) -> f64 {
        if self.candidate_calls == 0 { return 0.0; }
        self.total_candidates as f64 / self.candidate_calls as f64
    }

    fn to_ai_stats(&self, elapsed_ms: u128) -> AiStats {
        AiStats {
            nodes: self.nodes,
            pruned: self.pruned,
            avg_branch: self.avg_candidates(),
            time_ms: elapsed_ms,
        }
    }
}

// -------------------------------------------------------------------------
// Move undo system
// -------------------------------------------------------------------------

struct MoveUndo {
    x: usize,
    y: usize,
    player: Player,
    captured: Vec<(usize, usize)>,
}

fn apply_move(board: &mut Board, x: usize, y: usize, player: Player) -> MoveUndo {
    let opp = player.next().to_cell();
    let own = player.to_cell();
    board.cells[y][x] = own;

    let mut captured = Vec::new();

    for &(dx, dy) in DIRECTIONS.iter() {
        for dir in [1isize, -1isize] {
            let p1 = (x as isize + dx * dir,     y as isize + dy * dir);
            let p2 = (x as isize + dx * dir * 2, y as isize + dy * dir * 2);
            let p3 = (x as isize + dx * dir * 3, y as isize + dy * dir * 3);

            if !in_bounds(p1.0, p1.1) || !in_bounds(p2.0, p2.1) || !in_bounds(p3.0, p3.1) {
                continue;
            }

            if board.cells[p1.1 as usize][p1.0 as usize] == opp
                && board.cells[p2.1 as usize][p2.0 as usize] == opp
                && board.cells[p3.1 as usize][p3.0 as usize] == own
            {
                board.cells[p1.1 as usize][p1.0 as usize] = Cell::Empty;
                board.cells[p2.1 as usize][p2.0 as usize] = Cell::Empty;
                captured.push((p1.0 as usize, p1.1 as usize));
                captured.push((p2.0 as usize, p2.1 as usize));
            }
        }
    }

    MoveUndo { x, y, player, captured }
}

fn undo_move(board: &mut Board, undo: &MoveUndo) {
    board.cells[undo.y][undo.x] = Cell::Empty;
    let opp = undo.player.next().to_cell();
    for &(cx, cy) in &undo.captured {
        board.cells[cy][cx] = opp;
    }
}

// -------------------------------------------------------------------------
// Public entry point
// -------------------------------------------------------------------------

pub fn best_move(
    board: &Board,
    ai_player: Player,
    depth: u32,
    ai_captures: u32,
    opp_captures: u32,
) -> Result<((usize, usize), AiStats), String> {
    let timer = std::time::Instant::now();
    let mut stats = SearchStats::default();

    let candidates = candidate_moves(board);
    if candidates.is_empty() {
        let ai_stats = stats.to_ai_stats(timer.elapsed().as_millis());
        return Ok(((BOARD_SIZE / 2, BOARD_SIZE / 2), ai_stats));
    }

    let mut board = *board; // ← move this up, before order_moves
    let ordered = order_moves(&mut board, &candidates, ai_player);
    let mut best_score = i32::MIN;
    let mut best: Option<(usize, usize)> = None;

    for (x, y) in ordered {
        if board.is_double_three(x, y, ai_player) { continue; }

        let undo = apply_move(&mut board, x, y, ai_player);
        let new_ai_caps = ai_captures + undo.captured.len() as u32;

        if new_ai_caps >= CAPTURE_WIN || board.check_winner_from(x, y).is_some() {
            undo_move(&mut board, &undo);
            let ai_stats = stats.to_ai_stats(timer.elapsed().as_millis());
            return Ok(((x, y), ai_stats));
        }

        let mut score = minimax(
            &mut board, depth - 1, i32::MIN, i32::MAX,
            false, ai_player, new_ai_caps, opp_captures,
            &mut stats,
        );

        let noise = (rand::random::<f32>() * 2.0 - 1.0) * 10.;
        score += noise as i32;

        undo_move(&mut board, &undo);

        if score > best_score {
            best_score = score;
            best = Some((x, y));
        }
    }

    let ai_stats = stats.to_ai_stats(timer.elapsed().as_millis());
    best.map(|b| (b, ai_stats.clone()))
        .ok_or_else(|| "No legal moves available".to_string())
}

// -------------------------------------------------------------------------
// Minimax with alpha-beta pruning
// -------------------------------------------------------------------------

fn minimax(
    board: &mut Board,
    depth: u32,
    mut alpha: i32,
    mut beta: i32,
    is_maximizing: bool,
    ai_player: Player,
    ai_captures: u32,
    opp_captures: u32,
    stats: &mut SearchStats,
) -> i32 {
    stats.nodes += 1;

    if depth == 0 {
        return evaluate(board, ai_player, ai_captures, opp_captures);
    }

    let current = if is_maximizing { ai_player } else { ai_player.next() };
    let candidates = candidate_moves(board);

    stats.total_candidates += candidates.len() as u64;
    stats.candidate_calls += 1;

    if candidates.is_empty() {
        return evaluate(board, ai_player, ai_captures, opp_captures);
    }

    let ordered = order_moves(board, &candidates, current);

    if is_maximizing {
        let mut max_score = -INF;

        for (x, y) in ordered {
            let undo = apply_move(board, x, y, current);
            let new_ai_caps = ai_captures + undo.captured.len() as u32;

            let score = if new_ai_caps >= CAPTURE_WIN {
                WIN_SCORE + depth as i32
            } else if let Some(winner) = board.check_winner_from(x, y) {
                if winner == ai_player { WIN_SCORE + depth as i32 } else { -WIN_SCORE - depth as i32 }
            } else {
                minimax(board, depth - 1, alpha, beta, false, ai_player, new_ai_caps, opp_captures, stats)
            };

            undo_move(board, &undo);

            max_score = max_score.max(score);
            alpha = alpha.max(score);
            if beta <= alpha { stats.pruned += 1; break; }
        }

        max_score
    } else {
        let mut min_score = INF;

        for (x, y) in ordered {
            let undo = apply_move(board, x, y, current);
            let new_opp_caps = opp_captures + undo.captured.len() as u32;

            let score = if new_opp_caps >= CAPTURE_WIN {
                -WIN_SCORE - depth as i32
            } else if let Some(winner) = board.check_winner_from(x, y) {
                if winner == ai_player { WIN_SCORE + depth as i32 } else { -WIN_SCORE - depth as i32 }
            } else {
                minimax(board, depth - 1, alpha, beta, true, ai_player, ai_captures, new_opp_caps, stats)
            };

            undo_move(board, &undo);

            min_score = min_score.min(score);
            beta = beta.min(score);
            if beta <= alpha { stats.pruned += 1; break; }
        }

        min_score
    }
}

// -------------------------------------------------------------------------
// Candidate moves
// -------------------------------------------------------------------------

pub fn candidate_moves(board: &Board) -> Vec<(usize, usize)> {
    let mut seen = [[false; BOARD_SIZE]; BOARD_SIZE];
    let mut candidates = Vec::with_capacity(60);

    for y in 0..BOARD_SIZE {
        for x in 0..BOARD_SIZE {
            if board.cells[y][x] == Cell::Empty { continue; }
            for dy in -1isize..=1 {
                for dx in -1isize..=1 {
                    if dx == 0 && dy == 0 { continue; }
                    let nx = x as isize + dx;
                    let ny = y as isize + dy;
                    if !in_bounds(nx, ny) { continue; }
                    let (nx, ny) = (nx as usize, ny as usize);
                    if board.cells[ny][nx] != Cell::Empty { continue; }
                    if seen[ny][nx] { continue; }
                    seen[ny][nx] = true;
                    candidates.push((nx, ny));
                }
            }
        }
    }

    candidates
}

// -------------------------------------------------------------------------
// Move ordering
// -------------------------------------------------------------------------

fn order_moves(board: &mut Board, candidates: &[(usize, usize)], player: Player) -> Vec<(usize, usize)> {
    let mut scored: Vec<(i32, (usize, usize))> = candidates.iter()
        .map(|&(x, y)| (quick_score(board, x, y, player), (x, y)))
        .collect();
    scored.sort_unstable_by(|a, b| b.0.cmp(&a.0));
    scored.truncate(MAX_CANDIDATES);
    scored.into_iter().map(|(_, pos)| pos).collect()
}

fn quick_score(board: &mut Board, x: usize, y: usize, player: Player) -> i32 {
    let cell = player.to_cell();
    let opp  = player.next().to_cell();
    let mut score = 0;

    for &(dx, dy) in DIRECTIONS.iter() {
        // --- attack ---
        board.cells[y][x] = cell;
        let mut sx = x as isize;
        let mut sy = y as isize;
        while in_bounds(sx - dx, sy - dy)
            && board.cells[(sy - dy) as usize][(sx - dx) as usize] == cell
        {
            sx -= dx; sy -= dy;
        }
        score += score_line(board, sx as usize, sy as usize, dx, dy, cell);

        // --- block: try both directions to find opp sequence ---
        board.cells[y][x] = Cell::Empty;
        for &dir in &[1isize, -1isize] {
            let mut sx = x as isize;
            let mut sy = y as isize;
            while in_bounds(sx - dx * dir, sy - dy * dir)
                && board.cells[(sy - dy * dir) as usize][(sx - dx * dir) as usize] == opp
            {
                sx -= dx * dir;
                sy -= dy * dir;
            }
            if sx != x as isize || sy != y as isize {
                score += score_line(board, sx as usize, sy as usize, dx * dir, dy * dir, opp) / 2;
                break;
            }
        }
    }

    board.cells[y][x] = Cell::Empty;
    score += count_potential_captures(board, x, y, player) * 15_000;
    score
}

fn count_potential_captures(board: &Board, x: usize, y: usize, player: Player) -> i32 {
    let opp = player.next().to_cell();
    let own = player.to_cell();
    let mut captures = 0;

    for &(dx, dy) in DIRECTIONS.iter() {
        for dir in [1isize, -1isize] {
            let p1 = (x as isize + dx * dir,     y as isize + dy * dir);
            let p2 = (x as isize + dx * dir * 2, y as isize + dy * dir * 2);
            let p3 = (x as isize + dx * dir * 3, y as isize + dy * dir * 3);

            if !in_bounds(p1.0, p1.1) || !in_bounds(p2.0, p2.1) || !in_bounds(p3.0, p3.1) { continue; }

            if board.cells[p1.1 as usize][p1.0 as usize] == opp
                && board.cells[p2.1 as usize][p2.0 as usize] == opp
                && board.cells[p3.1 as usize][p3.0 as usize] == own
            {
                captures += 1;
            }
        }
    }
    captures
}

// -------------------------------------------------------------------------
// Heuristic evaluation
// -------------------------------------------------------------------------

fn evaluate(board: &Board, ai_player: Player, ai_captures: u32, opp_captures: u32) -> i32 {
    let mut score = 0;

    score += ai_captures as i32 * 10_000;
    score -= opp_captures as i32 * 10_000;

    if ai_captures  >= 8 { score += 500_000; }
    if opp_captures >= 8 { score -= 500_000; }

    score += score_patterns(board, ai_player);
    score -= score_patterns(board, ai_player.next());

    score
}

fn score_patterns(board: &Board, player: Player) -> i32 {
    let cell = player.to_cell();
    let mut score = 0;

    for &(dx, dy) in DIRECTIONS.iter() {
        for y in 0..BOARD_SIZE {
            for x in 0..BOARD_SIZE {
                if board.cells[y][x] != cell { continue; }

                let prev = (x as isize - dx, y as isize - dy);
                if in_bounds(prev.0, prev.1)
                    && board.cells[prev.1 as usize][prev.0 as usize] == cell
                {
                    continue;
                }

                score += score_line(board, x, y, dx, dy, cell);
            }
        }
    }

    score
}

fn score_line(board: &Board, x: usize, y: usize, dx: isize, dy: isize, cell: Cell) -> i32 {
    let mut count = 0;
    let mut nx = x as isize;
    let mut ny = y as isize;
    while in_bounds(nx, ny) && board.cells[ny as usize][nx as usize] == cell {
        count += 1;
        nx += dx;
        ny += dy;
    }

    if count == 0 { return 0; }

    let before = (x as isize - dx, y as isize - dy);
    let after  = (nx, ny);

    let open_before = in_bounds(before.0, before.1)
        && board.cells[before.1 as usize][before.0 as usize] == Cell::Empty;
    let open_after = in_bounds(after.0, after.1)
        && board.cells[after.1 as usize][after.0 as usize] == Cell::Empty;

    let open_ends = open_before as i32 + open_after as i32;

    match (count, open_ends) {
        (c, _) if c >= 5 => 900_000,
        (4, 2)           => 100_000,
        (4, 1)           => 10_000,
        (3, 2)           => 5_000,
        (3, 1)           => 500,
        (2, 2)           => 200,
        (2, 1)           => 50,
        (1, 2)           => 10,  // open single
        (1, 1)           => 0,   // half-open single — was 0, now gives variety
        _                => 0,
    }
}