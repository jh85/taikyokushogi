use crate::types::*;
use crate::pieces;
use crate::board::Board;
use crate::movegen::generate_legal_moves;
use crate::eval::{evaluate, MATE_SCORE};
use std::time::Instant;

pub struct SearchResult {
    pub best_move: Option<Move>,
    pub score: i32,
    pub nodes: u64,
    pub time_ms: u64,
}

pub fn search(board: &mut Board, depth: u32, time_limit_ms: u64) -> SearchResult {
    let start = Instant::now();
    let deadline = if time_limit_ms > 0 {
        Some(start + std::time::Duration::from_millis(time_limit_ms))
    } else {
        None
    };

    let mut best_move = None;
    let mut best_score = -MATE_SCORE - 1;
    let mut nodes: u64 = 0;

    let moves = generate_legal_moves(board);
    if moves.is_empty() {
        return SearchResult { best_move: None, score: evaluate(board), nodes: 1, time_ms: 0 };
    }

    // Sort moves for better pruning
    let mut scored_moves: Vec<(i32, usize)> = moves.iter().enumerate()
        .map(|(i, m)| (move_order_score(m), i))
        .collect();
    scored_moves.sort_by(|a, b| b.0.cmp(&a.0));

    for &(_, idx) in &scored_moves {
        let m = &moves[idx];
        board.apply_move(m);
        let score = -alphabeta(board, depth.saturating_sub(1), -MATE_SCORE - 1, -best_score.max(-MATE_SCORE - 1),
                               &mut nodes, deadline);
        board.undo_move();

        if score > best_score {
            best_score = score;
            best_move = Some(m.clone());
        }

        if let Some(dl) = deadline {
            if Instant::now() >= dl { break; }
        }
    }

    let elapsed = start.elapsed().as_millis() as u64;
    SearchResult { best_move, score: best_score, nodes, time_ms: elapsed }
}

fn alphabeta(board: &mut Board, depth: u32, mut alpha: i32, beta: i32,
             nodes: &mut u64, deadline: Option<Instant>) -> i32 {
    *nodes += 1;

    if *nodes % 50000 == 0 {
        if let Some(dl) = deadline {
            if Instant::now() >= dl { return alpha; }
        }
    }

    if let Some(result) = board.game_result() {
        return match result {
            GameResult::BlackWins => if board.side_to_move == BLACK { MATE_SCORE } else { -MATE_SCORE },
            GameResult::WhiteWins => if board.side_to_move == WHITE { MATE_SCORE } else { -MATE_SCORE },
            GameResult::Draw => 0,
        };
    }

    if depth == 0 {
        return evaluate(board);
    }

    let moves = generate_legal_moves(board);
    if moves.is_empty() {
        return evaluate(board);
    }

    // Simple move ordering
    let mut scored_moves: Vec<(i32, usize)> = moves.iter().enumerate()
        .map(|(i, m)| (move_order_score(m), i))
        .collect();
    scored_moves.sort_by(|a, b| b.0.cmp(&a.0));

    for &(_, idx) in &scored_moves {
        let m = &moves[idx];
        board.apply_move(m);
        let score = -alphabeta(board, depth - 1, -beta, -alpha, nodes, deadline);
        board.undo_move();

        if score > alpha {
            alpha = score;
        }
        if alpha >= beta {
            break;
        }
    }
    alpha
}

fn move_order_score(m: &Move) -> i32 {
    let mut score = 0i32;
    if m.captured_piece != 0 {
        score += pieces::value(m.captured_piece) * 10;
    }
    if m.promotion { score += 5000; }
    if m.mid_piece != 0 { score += pieces::value(m.mid_piece) * 5; }
    if let Some(ref caps) = m.range_caps {
        for &(_, pt, _) in caps {
            score += pieces::value(pt) * 10;
        }
    }
    score
}
