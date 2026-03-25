//! Example: play a random game of Taikyoku Shogi and display the result.
//!
//! ```sh
//! cargo run --example play
//! ```

use taikyoku_core::{Board, Color};

fn main() {
    let mut board = Board::initial();

    println!("=== Taikyoku Shogi ===");
    println!("Board: 36x36, {} squares", taikyoku_core::BOARD_SIZE * taikyoku_core::BOARD_SIZE);
    println!("Black pieces: {}", board.piece_count(Color::Black));
    println!("White pieces: {}", board.piece_count(Color::White));
    println!();

    // Count legal moves from the starting position
    let moves = board.legal_moves();
    println!("{} legal moves from starting position", moves.len());
    println!();

    // Look up piece info
    if let Some(info) = taikyoku_core::piece_info("K") {
        println!("King: value={}, slides={} dirs, royal=true", info.value, info.slide_directions);
    }
    if let Some(info) = taikyoku_core::piece_info("LN") {
        println!("Lion: value={}, area={} steps, igui={}", info.value, info.area_steps, info.has_igui);
    }
    println!();

    // Play a random game
    println!("Playing random game (max 500 moves)...");
    let mut move_count = 0;
    loop {
        if move_count >= 500 { break; }
        if let Some(result) = board.game_result() {
            println!("Game over at move {}: {}", move_count, result);
            break;
        }
        let mv = match board.random_move() {
            Some(m) => m,
            None => { println!("No legal moves at move {}", move_count); break; }
        };
        board.apply(&mv);
        move_count += 1;
    }
    println!("After {} moves: Black={}, White={}", move_count,
             board.piece_count(Color::Black), board.piece_count(Color::White));
    println!("Material score (Black's perspective): {}", board.material_score());
    println!();

    // Run a depth-1 search from the starting position
    let mut fresh = Board::initial();
    let result = fresh.search(1, 0);
    if let Some(mv) = result.best_move {
        println!("Depth-1 search: best={}, score={}, {} nodes in {}ms",
                 mv, result.score, result.nodes, result.time_ms);
    }
}
