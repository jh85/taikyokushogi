use crate::types::*;
use crate::pieces;
use crate::board::Board;

pub const MATE_SCORE: i32 = 1_000_000;

/// Material score from Black's perspective.
pub fn material_score(board: &Board) -> i32 {
    let mut score: i32 = 0;
    for c in 0..2 {
        let sign: i32 = if c == BLACK as usize { 1 } else { -1 };
        for i in 0..board.piece_list_len[c] {
            let sq = board.piece_list[c][i] as usize;
            if sq == INVALID_SQ as usize { continue; }
            let cell = board.cells[sq];
            if cell == EMPTY_CELL { continue; }
            score += sign * pieces::value(cell_piece(cell));
        }
    }
    score
}

/// Evaluate position from the side to move's perspective.
pub fn evaluate(board: &Board) -> i32 {
    if let Some(result) = board.game_result() {
        return match result {
            GameResult::BlackWins => if board.side_to_move == BLACK { MATE_SCORE } else { -MATE_SCORE },
            GameResult::WhiteWins => if board.side_to_move == WHITE { MATE_SCORE } else { -MATE_SCORE },
            GameResult::Draw => 0,
        };
    }
    let mat = material_score(board);
    if board.side_to_move == BLACK { mat } else { -mat }
}
