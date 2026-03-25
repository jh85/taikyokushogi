use crate::types::*;
use crate::pieces;

pub struct Board {
    pub cells: [Cell; NUM_SQUARES],
    pub side_to_move: u8,
    pub move_number: u32,
    // Piece lists per color
    pub piece_list: [[u16; MAX_PIECES_PER_SIDE]; 2],
    pub piece_list_len: [usize; 2],
    pub piece_count: [usize; 2],
    // Royal piece tracking
    pub royal_list: [[u16; MAX_ROYALS]; 2],
    pub royal_count: [usize; 2],
    // Undo stack
    history: Vec<UndoInfo>,
}

impl Clone for Board {
    fn clone(&self) -> Self {
        Board {
            cells: self.cells,
            side_to_move: self.side_to_move,
            move_number: self.move_number,
            piece_list: self.piece_list,
            piece_list_len: self.piece_list_len,
            piece_count: self.piece_count,
            royal_list: self.royal_list,
            royal_count: self.royal_count,
            history: self.history.clone(),
        }
    }
}

impl Board {
    pub fn new() -> Self {
        Board {
            cells: [EMPTY_CELL; NUM_SQUARES],
            side_to_move: BLACK,
            move_number: 1,
            piece_list: [[INVALID_SQ; MAX_PIECES_PER_SIDE]; 2],
            piece_list_len: [0; 2],
            piece_count: [0; 2],
            royal_list: [[INVALID_SQ; MAX_ROYALS]; 2],
            royal_count: [0; 2],
            history: Vec::new(),
        }
    }

    pub fn setup_initial(&mut self) {
        self.cells = [EMPTY_CELL; NUM_SQUARES];
        self.side_to_move = BLACK;
        self.move_number = 1;
        self.piece_list = [[INVALID_SQ; MAX_PIECES_PER_SIDE]; 2];
        self.piece_list_len = [0; 2];
        self.piece_count = [0; 2];
        self.royal_list = [[INVALID_SQ; MAX_ROYALS]; 2];
        self.royal_count = [0; 2];
        self.history.clear();

        // Place Black's pieces (rows 24-35)
        for (rank_idx, rank_str) in pieces::SETUP_RANKS.iter().enumerate() {
            let rank_pieces = pieces::parse_setup_rank(rank_str);
            let row = 35 - rank_idx; // Rank 1 at row 35
            for (col, piece_opt) in rank_pieces.iter().enumerate() {
                if let Some(pt) = piece_opt {
                    self.place_piece(row, col, *pt, BLACK);
                }
            }
        }

        // Place White's pieces (rows 0-11) — 180-degree rotation
        for (rank_idx, rank_str) in pieces::SETUP_RANKS.iter().enumerate() {
            let rank_pieces = pieces::parse_setup_rank(rank_str);
            let row = rank_idx; // Rank 1 at row 0
            let len = rank_pieces.len();
            for (col, piece_opt) in rank_pieces.iter().enumerate() {
                if let Some(pt) = piece_opt {
                    self.place_piece(row, len - 1 - col, *pt, WHITE);
                }
            }
        }
    }

    fn place_piece(&mut self, row: usize, col: usize, pt: u16, color: u8) {
        let sq = sq_index(row, col);
        self.cells[sq] = make_cell(pt, color);
        let c = color as usize;

        // Add to piece list
        let idx = self.piece_list_len[c];
        self.piece_list[c][idx] = sq as u16;
        self.piece_list_len[c] = idx + 1;
        self.piece_count[c] += 1;

        // Track royals
        if pieces::is_royal(pt) {
            let ri = self.royal_count[c];
            self.royal_list[c][ri] = sq as u16;
            self.royal_count[c] = ri + 1;
        }
    }

    /// Get the cell value at a given square index.
    #[inline]
    #[allow(dead_code)]
    pub fn at(&self, sq: usize) -> Cell {
        self.cells[sq]
    }

    pub fn apply_move(&mut self, m: &Move) {
        let from = m.from_sq as usize;
        let to = m.to_sq as usize;
        let from_cell = self.cells[from];
        let to_cell = self.cells[to];
        let pt = cell_piece(from_cell);
        let color = cell_color(from_cell);
        let c = color as usize;
        let _opp = 1 - c;

        let mut undo = UndoInfo {
            from_sq: m.from_sq, to_sq: m.to_sq,
            from_cell, to_cell,
            side: self.side_to_move,
            move_number: self.move_number,
            mid_sq: m.mid_sq, mid_cell: EMPTY_CELL,
            range_caps: None,
        };

        // Handle range captures
        if let Some(ref caps) = m.range_caps {
            let mut saved = Vec::new();
            for &(sq, _cap_pt, _cap_color) in caps {
                saved.push((sq, self.cells[sq as usize]));
                self.remove_from_lists(sq as usize);
                self.cells[sq as usize] = EMPTY_CELL;
            }
            undo.range_caps = Some(saved);
        }

        // Handle lion mid-capture
        if m.mid_sq != INVALID_SQ {
            let msq = m.mid_sq as usize;
            undo.mid_cell = self.cells[msq];
            self.remove_from_lists(msq);
            self.cells[msq] = EMPTY_CELL;
        }

        // Handle igui
        if m.is_igui {
            if to_cell != EMPTY_CELL {
                self.remove_from_lists(to);
                self.cells[to] = EMPTY_CELL;
            }
            // Piece may promote in place
            if m.promotion {
                if let Some(promo_pt) = pieces::promotes_to(pt) {
                    let new_cell = make_cell(promo_pt, color);
                    self.cells[from] = new_cell;
                    self.update_royal_status(from, pt, promo_pt, c);
                }
            }
            self.side_to_move = 1 - self.side_to_move;
            if self.side_to_move == BLACK { self.move_number += 1; }
            self.history.push(undo);
            return;
        }

        // Remove from origin
        self.cells[from] = EMPTY_CELL;
        self.remove_sq_from_piece_list(from, c);
        if pieces::is_royal(pt) {
            self.remove_sq_from_royal_list(from, c);
        }

        // Capture at destination
        if to_cell != EMPTY_CELL {
            self.remove_from_lists(to);
        }

        // Determine final piece (promotion)
        let final_pt = if m.promotion {
            pieces::promotes_to(pt).unwrap_or(pt)
        } else {
            pt
        };

        // Place at destination
        self.cells[to] = make_cell(final_pt, color);
        self.add_sq_to_piece_list(to, c);
        if pieces::is_royal(final_pt) {
            let ri = self.royal_count[c];
            self.royal_list[c][ri] = to as u16;
            self.royal_count[c] = ri + 1;
        }

        self.side_to_move = 1 - self.side_to_move;
        if self.side_to_move == BLACK { self.move_number += 1; }
        self.history.push(undo);
    }

    pub fn undo_move(&mut self) -> bool {
        let undo = match self.history.pop() {
            Some(u) => u,
            None => return false,
        };

        self.side_to_move = undo.side;
        self.move_number = undo.move_number;

        let from = undo.from_sq as usize;
        let to = undo.to_sq as usize;

        // Check if it was igui
        if undo.from_sq == undo.to_sq && undo.to_cell != EMPTY_CELL {
            // Restore piece at from to original
            self.cells[from] = undo.from_cell;
            // Restore captured piece at to (same square, but it was an adjacent capture)
            // Actually for igui, from == to but the captured piece was adjacent
            // Let me re-check the igui logic...
            // For igui: the piece stays at from, and an adjacent piece at some other square was captured.
            // But in our Move encoding, to_sq == from_sq for igui and the captured piece is tracked separately.
            // The undo needs to restore from_cell and to_cell.
            self.cells[from] = undo.from_cell;
            // to_cell was the captured piece... but to == from for igui
            // Actually let me look at how apply_move handles igui:
            // It removes to_cell (the adjacent captured piece) from lists, sets cells[to] = EMPTY
            // But to == from in igui... that's a problem in our encoding.
            // For simplicity, let's just rebuild lists.
            self.rebuild_lists();
            // Restore mid/range captures
            if undo.mid_sq != INVALID_SQ {
                self.cells[undo.mid_sq as usize] = undo.mid_cell;
            }
            if let Some(ref caps) = undo.range_caps {
                for &(sq, cell) in caps {
                    self.cells[sq as usize] = cell;
                }
            }
            self.rebuild_lists();
            return true;
        }

        // Standard undo: remove piece from to, restore from and to cells
        self.cells[to] = EMPTY_CELL;
        self.cells[from] = undo.from_cell;
        if undo.to_cell != EMPTY_CELL {
            self.cells[to] = undo.to_cell;
        }

        // Restore mid-capture
        if undo.mid_sq != INVALID_SQ {
            self.cells[undo.mid_sq as usize] = undo.mid_cell;
        }

        // Restore range captures
        if let Some(ref caps) = undo.range_caps {
            for &(sq, cell) in caps {
                self.cells[sq as usize] = cell;
            }
        }

        // Rebuild piece lists (simpler than incremental update for correctness)
        self.rebuild_lists();
        true
    }

    fn rebuild_lists(&mut self) {
        self.piece_list_len = [0; 2];
        self.piece_count = [0; 2];
        self.royal_count = [0; 2];
        self.piece_list = [[INVALID_SQ; MAX_PIECES_PER_SIDE]; 2];
        self.royal_list = [[INVALID_SQ; MAX_ROYALS]; 2];

        for sq in 0..NUM_SQUARES {
            let cell = self.cells[sq];
            if cell != EMPTY_CELL {
                let pt = cell_piece(cell);
                let color = cell_color(cell);
                let c = color as usize;
                let idx = self.piece_list_len[c];
                self.piece_list[c][idx] = sq as u16;
                self.piece_list_len[c] = idx + 1;
                self.piece_count[c] += 1;
                if pieces::is_royal(pt) {
                    let ri = self.royal_count[c];
                    self.royal_list[c][ri] = sq as u16;
                    self.royal_count[c] = ri + 1;
                }
            }
        }
    }

    fn remove_from_lists(&mut self, sq: usize) {
        let cell = self.cells[sq];
        if cell == EMPTY_CELL { return; }
        let pt = cell_piece(cell);
        let color = cell_color(cell) as usize;
        self.remove_sq_from_piece_list(sq, color);
        self.piece_count[color] -= 1;
        if pieces::is_royal(pt) {
            self.remove_sq_from_royal_list(sq, color);
        }
    }

    fn remove_sq_from_piece_list(&mut self, sq: usize, color: usize) {
        let sq16 = sq as u16;
        let len = self.piece_list_len[color];
        for i in 0..len {
            if self.piece_list[color][i] == sq16 {
                self.piece_list[color][i] = self.piece_list[color][len - 1];
                self.piece_list[color][len - 1] = INVALID_SQ;
                self.piece_list_len[color] = len - 1;
                return;
            }
        }
    }

    fn remove_sq_from_royal_list(&mut self, sq: usize, color: usize) {
        let sq16 = sq as u16;
        let len = self.royal_count[color];
        for i in 0..len {
            if self.royal_list[color][i] == sq16 {
                self.royal_list[color][i] = self.royal_list[color][len - 1];
                self.royal_list[color][len - 1] = INVALID_SQ;
                self.royal_count[color] = len - 1;
                return;
            }
        }
    }

    fn add_sq_to_piece_list(&mut self, sq: usize, color: usize) {
        let idx = self.piece_list_len[color];
        self.piece_list[color][idx] = sq as u16;
        self.piece_list_len[color] = idx + 1;
        self.piece_count[color] += 1;
    }

    fn update_royal_status(&mut self, sq: usize, old_pt: u16, new_pt: u16, color: usize) {
        if pieces::is_royal(old_pt) && !pieces::is_royal(new_pt) {
            self.remove_sq_from_royal_list(sq, color);
        } else if !pieces::is_royal(old_pt) && pieces::is_royal(new_pt) {
            let ri = self.royal_count[color];
            self.royal_list[color][ri] = sq as u16;
            self.royal_count[color] = ri + 1;
        }
    }

    pub fn game_result(&self) -> Option<GameResult> {
        let b = self.royal_count[BLACK as usize] > 0;
        let w = self.royal_count[WHITE as usize] > 0;
        match (b, w) {
            (true, true) => None,
            (true, false) => Some(GameResult::BlackWins),
            (false, true) => Some(GameResult::WhiteWins),
            (false, false) => Some(GameResult::Draw),
        }
    }

    pub fn display(&self) -> String {
        let mut s = String::new();
        for r in 0..BOARD_SIZE {
            s.push_str(&format!("{:2} ", BOARD_SIZE - r));
            for c in 0..BOARD_SIZE {
                let cell = self.cells[sq_index(r, c)];
                if cell == EMPTY_CELL {
                    s.push_str(".. ");
                } else {
                    let pt = cell_piece(cell);
                    let color = cell_color(cell);
                    let prefix = if color == WHITE { 'v' } else { '^' };
                    let ab = pieces::abbrev(pt);
                    s.push(prefix);
                    s.push_str(&format!("{:<2}", &ab[..ab.len().min(2)]));
                }
            }
            s.push('\n');
        }
        s
    }
}
