//! TSFEN notation for Taikyoku Shogi positions.
//!
//! Format: `rank/rank/.../rank side move_number`
//!
//! - 36 ranks separated by `/`, from row 0 (top) to row 35 (bottom)
//! - Each rank lists pieces left to right; empty squares use numbers (like FEN)
//! - Black pieces: uppercase; White pieces: lowercase
//! - Single-char abbreviations: bare letter (e.g., `P`, `K`, `r`, `b`)
//! - Multi-char abbreviations: wrapped in parentheses (e.g., `(CP)`, `(dh)`)
//! - Side to move: `b` (Black) or `w` (White)
//! - Move number: integer starting at 1

use crate::types::*;
use crate::pieces;
use crate::board::Board;

/// Encode a board position as a TSFEN string.
pub fn to_tsfen(board: &Board) -> String {
    let mut ranks = Vec::with_capacity(BOARD_SIZE);
    for r in 0..BOARD_SIZE {
        ranks.push(encode_rank(board, r));
    }
    let side = if board.side_to_move == BLACK { 'b' } else { 'w' };
    format!("{} {} {}", ranks.join("/"), side, board.move_number)
}

/// Parse a TSFEN string into a Board. Returns `Err` with a message on failure.
pub fn from_tsfen(tsfen: &str) -> Result<Board, String> {
    let parts: Vec<&str> = tsfen.trim().splitn(3, ' ').collect();
    let rank_str = parts.first().ok_or("empty TSFEN")?;
    let side = parts.get(1).copied().unwrap_or("b");
    let move_num: u32 = parts.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let mut board = Board::new();
    board.side_to_move = if side == "w" { WHITE } else { BLACK };
    board.move_number = move_num;

    let ranks: Vec<&str> = rank_str.split('/').collect();
    if ranks.len() != BOARD_SIZE {
        return Err(format!("expected {} ranks, got {}", BOARD_SIZE, ranks.len()));
    }

    for (r, rank) in ranks.iter().enumerate() {
        decode_rank(&mut board, r, rank)?;
    }

    board.rebuild_lists_pub();
    Ok(board)
}

fn encode_rank(board: &Board, row: usize) -> String {
    let mut parts = String::new();
    let mut empty = 0u32;

    for c in 0..BOARD_SIZE {
        let cell = board.cells[row * BOARD_SIZE + c];
        if cell == EMPTY_CELL {
            empty += 1;
        } else {
            if empty > 0 {
                parts.push_str(&empty.to_string());
                empty = 0;
            }
            let pt = cell_piece(cell);
            let color = cell_color(cell);
            let abbrev = pieces::abbrev(pt);
            encode_piece(&mut parts, abbrev, color);
        }
    }
    if empty > 0 {
        parts.push_str(&empty.to_string());
    }
    parts
}

fn encode_piece(out: &mut String, abbrev: &str, color: u8) {
    // Black = uppercase, White = lowercase
    let s: String = if color == BLACK {
        abbrev.to_uppercase()
    } else {
        abbrev.to_lowercase()
    };

    if abbrev.len() == 1 {
        out.push_str(&s);
    } else {
        out.push('(');
        out.push_str(&s);
        out.push(')');
    }
}

fn decode_rank(board: &mut Board, row: usize, rank_str: &str) -> Result<(), String> {
    let bytes = rank_str.as_bytes();
    let n = bytes.len();
    let mut col = 0usize;
    let mut i = 0usize;

    while i < n && col < BOARD_SIZE {
        let ch = bytes[i] as char;

        if ch.is_ascii_digit() {
            // Empty squares
            let start = i;
            while i < n && (bytes[i] as char).is_ascii_digit() {
                i += 1;
            }
            let count: usize = rank_str[start..i].parse()
                .map_err(|_| format!("invalid number in rank {}", row))?;
            col += count;
        } else if ch == '(' {
            // Multi-character abbreviation
            i += 1; // skip '('
            let start = i;
            while i < n && bytes[i] != b')' {
                i += 1;
            }
            if i >= n {
                return Err(format!("unclosed parenthesis in rank {}", row));
            }
            let abbrev_raw = &rank_str[start..i];
            i += 1; // skip ')'

            let (color, abbrev) = parse_piece_token(abbrev_raw);
            if let Some(pt) = pieces::find_by_abbrev(&abbrev) {
                let sq = row * BOARD_SIZE + col;
                board.cells[sq] = make_cell(pt, color);
            }
            col += 1;
        } else {
            // Single character piece
            let color = if ch.is_ascii_uppercase() { BLACK } else { WHITE };
            let abbrev = ch.to_ascii_uppercase().to_string();
            if let Some(pt) = pieces::find_by_abbrev(&abbrev) {
                let sq = row * BOARD_SIZE + col;
                board.cells[sq] = make_cell(pt, color);
            }
            col += 1;
            i += 1;
        }
    }
    Ok(())
}

/// Parse a piece token (from inside parens or single char) into (color, uppercase_abbrev).
fn parse_piece_token(token: &str) -> (u8, String) {
    let first_alpha = token.chars().find(|c| c.is_ascii_alphabetic());
    let color = match first_alpha {
        Some(c) if c.is_ascii_uppercase() => BLACK,
        _ => WHITE,
    };
    (color, token.to_ascii_uppercase())
}
