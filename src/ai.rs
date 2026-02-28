use crate::game::{Board, Player, BOARD_SIZE};

use std::thread;
use std::time::Duration;
use rand::Rng;

pub fn best_move(board: &Board, ai_player: Player, depth: u32) -> Result<(usize, usize), String> {
    let mut rng = rand::thread_rng();

    let delay_ms = rng.gen_range(0..2000);
    thread::sleep(Duration::from_millis(delay_ms));

    let available: Vec<(usize, usize)> = (0..BOARD_SIZE)
        .flat_map(|y| (0..BOARD_SIZE).map(move |x| (x, y)))
        .filter(|&(x, y)| board.is_empty(x, y))
        .collect();

    if available.is_empty() {
        return Err("No move found".to_string());
    }
    return Ok(available[0]);

    let idx = rng.gen_range(0..available.len());
    Ok(available[idx])
}