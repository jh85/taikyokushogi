/// Core types for Taikyoku Shogi engine.

pub const BOARD_SIZE: usize = 36;
pub const NUM_SQUARES: usize = BOARD_SIZE * BOARD_SIZE; // 1296
pub const BLACK: u8 = 0;
pub const WHITE: u8 = 1;

/// Promotion zone depth (11 rows on opponent's side).
pub const PROMO_ZONE_DEPTH: usize = 11;

/// Check if a square is in the promotion zone for the given color.
/// Black's promotion zone: rows 0..=10 (opponent's side).
/// White's promotion zone: rows 25..=35 (opponent's side).
#[inline]
pub fn in_promo_zone(sq: usize, color: u8) -> bool {
    let row = sq / BOARD_SIZE;
    if color == BLACK { row < PROMO_ZONE_DEPTH } else { row >= BOARD_SIZE - PROMO_ZONE_DEPTH }
}

/// Check if a square is the farthest rank for the given color.
/// Black: row 0. White: row 35.
#[inline]
pub fn is_farthest_rank(sq: usize, color: u8) -> bool {
    let row = sq / BOARD_SIZE;
    if color == BLACK { row == 0 } else { row == BOARD_SIZE - 1 }
}

// Directions (from Black's perspective: forward = decreasing row)
pub const N: usize = 0;
pub const NE: usize = 1;
pub const E: usize = 2;
pub const SE: usize = 3;
pub const S: usize = 4;
pub const SW: usize = 5;
pub const W: usize = 6;
pub const NW: usize = 7;
pub const NUM_DIRS: usize = 8;

// Direction deltas for Black
pub const DIR_DR: [i32; 8] = [-1, -1, 0, 1, 1, 1, 0, -1];
pub const DIR_DC: [i32; 8] = [0, 1, 1, 1, 0, -1, -1, -1];

/// Cell encoding: 0 = empty, otherwise (piece_type << 1) | color
pub type Cell = u16;
pub const EMPTY_CELL: Cell = 0;
pub const INVALID_SQ: u16 = 0xFFFF;

pub const MAX_PIECES_PER_SIDE: usize = 410;
pub const MAX_ROYALS: usize = 8;

#[inline]
pub fn make_cell(piece_type: u16, color: u8) -> Cell {
    (piece_type << 1) | (color as u16)
}

#[inline]
pub fn cell_piece(cell: Cell) -> u16 {
    cell >> 1
}

#[inline]
pub fn cell_color(cell: Cell) -> u8 {
    (cell & 1) as u8
}

#[inline]
pub fn sq_index(row: usize, col: usize) -> usize {
    row * BOARD_SIZE + col
}

#[inline]
pub fn sq_row(sq: usize) -> usize {
    sq / BOARD_SIZE
}

#[inline]
pub fn sq_col(sq: usize) -> usize {
    sq % BOARD_SIZE
}

/// Get direction deltas adjusted for color.
#[inline]
pub fn get_deltas(dir: usize, color: u8) -> (i32, i32) {
    if color == BLACK {
        (DIR_DR[dir], DIR_DC[dir])
    } else {
        (-DIR_DR[dir], -DIR_DC[dir])
    }
}

/// Step in a direction from a square. Returns None if out of bounds.
#[inline]
pub fn step_sq(sq: usize, dir: usize, color: u8) -> Option<usize> {
    let (dr, dc) = get_deltas(dir, color);
    let r = sq_row(sq) as i32 + dr;
    let c = sq_col(sq) as i32 + dc;
    if r >= 0 && r < BOARD_SIZE as i32 && c >= 0 && c < BOARD_SIZE as i32 {
        Some(r as usize * BOARD_SIZE + c as usize)
    } else {
        None
    }
}

#[derive(Clone, Copy, Debug)]
pub enum GameResult {
    BlackWins,
    WhiteWins,
    Draw,
}

/// Movement rules for a piece type.
#[derive(Clone, Debug)]
pub struct Movement {
    pub slides: Vec<(u8, u8)>,       // (direction, max_range) 0=unlimited
    pub jumps: Vec<(i8, i8)>,        // (delta_row, delta_col)
    pub hook: Option<HookType>,
    pub area: u8,                     // 0=none, 2=lion, 3=lion_dog
    pub range_capture: Vec<u8>,       // directions for range capture
    pub igui: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum HookType {
    Orthogonal,
    Diagonal,
}

impl Movement {
    pub fn empty() -> Self {
        Movement {
            slides: Vec::new(),
            jumps: Vec::new(),
            hook: None,
            area: 0,
            range_capture: Vec::new(),
            igui: false,
        }
    }
}

/// A move in the game.
#[derive(Clone, Debug)]
pub struct Move {
    pub from_sq: u16,
    pub to_sq: u16,
    pub promotion: bool,
    pub captured_piece: u16,   // 0 if no capture
    pub captured_color: u8,
    pub is_igui: bool,
    // For lion mid-capture
    pub mid_sq: u16,           // INVALID_SQ if none
    pub mid_piece: u16,
    pub mid_color: u8,
    // Range captures stored separately (rare)
    pub range_caps: Option<Vec<(u16, u16, u8)>>, // (sq, piece, color)
}

impl Move {
    pub fn simple(from: u16, to: u16) -> Self {
        Move {
            from_sq: from, to_sq: to, promotion: false,
            captured_piece: 0, captured_color: 0, is_igui: false,
            mid_sq: INVALID_SQ, mid_piece: 0, mid_color: 0,
            range_caps: None,
        }
    }
}

/// Piece rank for range-capture restrictions.
pub const RANK_ROYAL: u8 = 0;
pub const RANK_GREAT: u8 = 1;
pub const RANK_VICE: u8 = 2;
pub const RANK_RANGE_CAP: u8 = 3;
pub const RANK_NORMAL: u8 = 4;

/// No-progress draw threshold (in plies). If 1000 consecutive plies pass
/// with no capture and no promotion, the game is automatically drawn.
pub const DRAW_PLIES: u32 = 1000;

/// Undo information saved per move.
#[derive(Clone)]
pub struct UndoInfo {
    pub from_sq: u16,
    pub to_sq: u16,
    pub from_cell: Cell,
    pub to_cell: Cell,
    pub side: u8,
    pub move_number: u32,
    pub mid_sq: u16,
    pub mid_cell: Cell,
    pub range_caps: Option<Vec<(u16, Cell)>>,
    pub no_progress_plies: u32,
}

// ============================================================
// Precomputed ray table
// ============================================================
/// For each (square, direction), the list of squares along that ray.
/// Computed once at init.
pub struct RayTable {
    // rays[sq * 8 + dir] = start index into `data`
    // lens[sq * 8 + dir] = length
    offsets: Vec<u32>,
    lens: Vec<u8>,
    data: Vec<u16>,
}

impl RayTable {
    pub fn new() -> Self {
        let total = NUM_SQUARES * NUM_DIRS;
        let mut offsets = vec![0u32; total];
        let mut lens = vec![0u8; total];
        let mut data = Vec::with_capacity(total * 18); // avg ~18 squares per ray

        for sq in 0..NUM_SQUARES {
            let r = sq_row(sq) as i32;
            let c = sq_col(sq) as i32;
            for dir in 0..NUM_DIRS {
                let idx = sq * NUM_DIRS + dir;
                offsets[idx] = data.len() as u32;
                let dr = DIR_DR[dir];
                let dc = DIR_DC[dir];
                let mut cr = r + dr;
                let mut cc = c + dc;
                let mut count = 0u8;
                while cr >= 0 && cr < BOARD_SIZE as i32 && cc >= 0 && cc < BOARD_SIZE as i32 {
                    data.push((cr as usize * BOARD_SIZE + cc as usize) as u16);
                    count += 1;
                    cr += dr;
                    cc += dc;
                }
                lens[idx] = count;
            }
        }
        RayTable { offsets, lens, data }
    }

    /// Get the ray from `sq` in direction `dir`.
    #[inline]
    pub fn ray(&self, sq: usize, dir: usize) -> &[u16] {
        let idx = sq * NUM_DIRS + dir;
        let start = self.offsets[idx] as usize;
        let len = self.lens[idx] as usize;
        &self.data[start..start + len]
    }

    /// Get ray adjusted for color (Black uses dir directly, White uses opposite).
    #[inline]
    pub fn ray_for_color(&self, sq: usize, dir: usize, color: u8) -> &[u16] {
        if color == BLACK {
            self.ray(sq, dir)
        } else {
            self.ray(sq, (dir + 4) % 8)
        }
    }
}

use std::sync::OnceLock;
static RAY_TABLE: OnceLock<RayTable> = OnceLock::new();

pub fn ray_table() -> &'static RayTable {
    RAY_TABLE.get_or_init(RayTable::new)
}
