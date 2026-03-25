//! # Taikyoku Shogi Engine
//!
//! A complete engine for [Taikyoku Shogi](https://en.wikipedia.org/wiki/Taikyoku_shogi),
//! the largest known variant of shogi (Japanese chess).
//!
//! - **36 x 36** board (1,296 squares)
//! - **804 pieces** (402 per side)
//! - **209 piece types** with distinct movement patterns
//!
//! ## Quick Start
//!
//! ```rust
//! use taikyokushogi::Board;
//!
//! let mut board = Board::initial();
//! assert_eq!(board.piece_count(taikyokushogi::Color::Black), 402);
//!
//! let moves = board.legal_moves();
//! println!("{} legal moves from starting position", moves.len());
//!
//! // Apply the first move
//! board.apply(&moves[0]);
//! println!("Score: {}", board.material_score());
//!
//! // Undo
//! board.undo();
//! ```
//!
//! ## Search
//!
//! ```rust,no_run
//! use taikyokushogi::Board;
//!
//! let mut board = Board::initial();
//! let result = board.search(2, 5000); // depth 2, 5s time limit
//! if let Some(mv) = result.best_move {
//!     println!("Best move: {}, score: {}", mv, result.score);
//! }
//! ```

mod types;
mod pieces;
mod board;
mod movegen;
mod eval;
mod search;

#[cfg(feature = "python")]
mod python;

// ============================================================
// Public re-exports — the user-facing API
// ============================================================

/// Board size (36).
pub const BOARD_SIZE: usize = types::BOARD_SIZE;

/// Number of squares (1,296).
pub const NUM_SQUARES: usize = types::NUM_SQUARES;

/// Side / player color.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    Black = 0,
    White = 1,
}

impl Color {
    /// The other color.
    pub fn opponent(self) -> Self {
        match self {
            Color::Black => Color::White,
            Color::White => Color::Black,
        }
    }

    pub(crate) fn raw(self) -> u8 {
        self as u8
    }

    pub(crate) fn from_raw(v: u8) -> Self {
        if v == 0 { Color::Black } else { Color::White }
    }
}

impl std::fmt::Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Color::Black => write!(f, "Black"),
            Color::White => write!(f, "White"),
        }
    }
}

/// Result of a finished game.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameResult {
    BlackWins,
    WhiteWins,
    Draw,
}

impl std::fmt::Display for GameResult {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            GameResult::BlackWins => write!(f, "Black wins"),
            GameResult::WhiteWins => write!(f, "White wins"),
            GameResult::Draw => write!(f, "Draw"),
        }
    }
}

/// A square on the 36x36 board, represented as `(row, col)`.
///
/// - Row 0 = top of the board (White's back rank)
/// - Row 35 = bottom (Black's back rank)
/// - Col 0 = left, Col 35 = right
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Square {
    pub row: usize,
    pub col: usize,
}

impl Square {
    pub fn new(row: usize, col: usize) -> Self {
        Square { row, col }
    }

    pub(crate) fn from_index(idx: usize) -> Self {
        Square { row: idx / BOARD_SIZE, col: idx % BOARD_SIZE }
    }

    /// Convert to flat index (row * 36 + col).
    pub fn index(&self) -> usize {
        self.row * BOARD_SIZE + self.col
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({},{})", self.row, self.col)
    }
}

/// A piece on the board.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Piece {
    /// Internal piece type ID.
    pub(crate) type_id: u16,
    /// Color of this piece.
    pub color: Color,
}

impl Piece {
    /// Abbreviation (e.g., `"K"`, `"CP"`, `"GG"`).
    pub fn abbrev(&self) -> &'static str {
        pieces::abbrev(self.type_id)
    }

    /// Full English name.
    pub fn name(&self) -> &'static str {
        pieces::name(self.type_id)
    }

    /// Material value.
    pub fn value(&self) -> i32 {
        pieces::value(self.type_id)
    }

    /// Whether this is a royal piece (King or Crown Prince).
    pub fn is_royal(&self) -> bool {
        pieces::is_royal(self.type_id)
    }

    /// What this piece promotes to, if anything.
    pub fn promotes_to(&self) -> Option<&'static str> {
        pieces::promotes_to(self.type_id).map(|pt| pieces::abbrev(pt))
    }
}

impl std::fmt::Display for Piece {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let prefix = if self.color == Color::White { 'v' } else { '^' };
        write!(f, "{}{}", prefix, self.abbrev())
    }
}

/// A move in the game.
#[derive(Debug, Clone)]
pub struct Move {
    inner: types::Move,
}

impl Move {
    /// Source square.
    pub fn from(&self) -> Square {
        Square::from_index(self.inner.from_sq as usize)
    }

    /// Destination square.
    pub fn to(&self) -> Square {
        Square::from_index(self.inner.to_sq as usize)
    }

    /// Whether this move promotes the piece.
    pub fn is_promotion(&self) -> bool {
        self.inner.promotion
    }

    /// Whether this is an igui (capture without moving).
    pub fn is_igui(&self) -> bool {
        self.inner.is_igui
    }

    /// The captured piece's abbreviation, if any.
    pub fn captured(&self) -> Option<&'static str> {
        if self.inner.captured_piece != 0 {
            Some(pieces::abbrev(self.inner.captured_piece))
        } else {
            None
        }
    }

    /// Access the internal move representation.
    pub fn raw(&self) -> &types::Move {
        &self.inner
    }
}

impl std::fmt::Display for Move {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}->{}", self.from(), self.to())?;
        if self.is_promotion() { write!(f, "+")?; }
        Ok(())
    }
}

/// Result of a search.
pub struct SearchResult {
    /// Best move found, if any.
    pub best_move: Option<Move>,
    /// Evaluation score (positive = good for side to move).
    pub score: i32,
    /// Number of nodes searched.
    pub nodes: u64,
    /// Wall-clock time in milliseconds.
    pub time_ms: u64,
}

/// Information about a piece type.
pub struct PieceInfo {
    pub abbrev: &'static str,
    pub name: &'static str,
    pub value: i32,
    pub promotes_to: Option<&'static str>,
    pub slide_directions: usize,
    pub jump_destinations: usize,
    pub has_hook: bool,
    pub area_steps: u8,
    pub has_range_capture: bool,
    pub has_igui: bool,
}

/// Look up information about a piece type by abbreviation.
///
/// ```rust
/// let info = taikyokushogi::piece_info("K").unwrap();
/// assert_eq!(info.name, "King");
/// assert!(info.value > 0);
/// ```
pub fn piece_info(abbrev: &str) -> Option<PieceInfo> {
    let pt = pieces::find_by_abbrev(abbrev)?;
    let mv = pieces::movement(pt);
    let promo = pieces::promotes_to(pt).map(|p| pieces::abbrev(p));
    Some(PieceInfo {
        abbrev: pieces::abbrev(pt),
        name: pieces::name(pt),
        value: pieces::value(pt),
        promotes_to: promo,
        slide_directions: mv.slides.len(),
        jump_destinations: mv.jumps.len(),
        has_hook: mv.hook.is_some(),
        area_steps: mv.area,
        has_range_capture: !mv.range_capture.is_empty(),
        has_igui: mv.igui,
    })
}

/// Total number of defined piece types (base + promoted).
pub fn num_piece_types() -> usize {
    pieces::num_piece_types()
}

// ============================================================
// Board — the main game interface
// ============================================================

/// The Taikyoku Shogi board.
///
/// This is the primary type for interacting with the engine.
/// Create one with [`Board::initial()`] for the starting position,
/// or [`Board::empty()`] for an empty board.
pub struct Board {
    inner: board::Board,
}

impl Board {
    /// Create an empty board.
    pub fn empty() -> Self {
        Board { inner: board::Board::new() }
    }

    /// Create a board with the standard initial position (804 pieces).
    ///
    /// ```rust
    /// let board = taikyokushogi::Board::initial();
    /// assert_eq!(board.piece_count(taikyokushogi::Color::Black), 402);
    /// assert_eq!(board.piece_count(taikyokushogi::Color::White), 402);
    /// ```
    pub fn initial() -> Self {
        let mut b = board::Board::new();
        b.setup_initial();
        Board { inner: b }
    }

    /// Whose turn is it?
    pub fn side_to_move(&self) -> Color {
        Color::from_raw(self.inner.side_to_move)
    }

    /// Current full-move number (starts at 1, increments after White moves).
    pub fn move_number(&self) -> u32 {
        self.inner.move_number
    }

    /// Get the piece at `(row, col)`, or `None` if empty.
    pub fn get(&self, row: usize, col: usize) -> Option<Piece> {
        let sq = types::sq_index(row, col);
        let cell = self.inner.cells[sq];
        if cell == types::EMPTY_CELL {
            None
        } else {
            Some(Piece {
                type_id: types::cell_piece(cell),
                color: Color::from_raw(types::cell_color(cell)),
            })
        }
    }

    /// Number of pieces for the given color.
    pub fn piece_count(&self, color: Color) -> usize {
        self.inner.piece_count[color.raw() as usize]
    }

    /// Material score from Black's perspective.
    /// Positive = Black has more material.
    pub fn material_score(&self) -> i32 {
        eval::material_score(&self.inner)
    }

    /// Evaluate the position from the side to move's perspective.
    pub fn evaluate(&self) -> i32 {
        eval::evaluate(&self.inner)
    }

    /// Check if the game is over.
    pub fn game_result(&self) -> Option<GameResult> {
        self.inner.game_result().map(|r| match r {
            types::GameResult::BlackWins => GameResult::BlackWins,
            types::GameResult::WhiteWins => GameResult::WhiteWins,
            types::GameResult::Draw => GameResult::Draw,
        })
    }

    /// Generate all legal moves for the side to move.
    pub fn legal_moves(&self) -> Vec<Move> {
        movegen::generate_legal_moves(&self.inner)
            .into_iter()
            .map(|m| Move { inner: m })
            .collect()
    }

    /// Apply a move to the board.
    pub fn apply(&mut self, mv: &Move) {
        self.inner.apply_move(&mv.inner);
    }

    /// Apply a move specified by coordinates. Returns `true` if a matching legal
    /// move was found and applied, `false` otherwise.
    ///
    /// ```rust,no_run
    /// let mut board = taikyokushogi::Board::initial();
    /// board.apply_by_coord(25, 0, 24, 0, false); // move pawn forward
    /// ```
    pub fn apply_by_coord(&mut self, from_row: usize, from_col: usize,
                          to_row: usize, to_col: usize, promotion: bool) -> bool {
        let from_sq = types::sq_index(from_row, from_col) as u16;
        let to_sq = types::sq_index(to_row, to_col) as u16;
        let moves = movegen::generate_legal_moves(&self.inner);
        for m in &moves {
            if m.from_sq == from_sq && m.to_sq == to_sq && m.promotion == promotion {
                self.inner.apply_move(m);
                return true;
            }
        }
        false
    }

    /// Undo the last move. Returns `false` if there is nothing to undo.
    pub fn undo(&mut self) -> bool {
        self.inner.undo_move()
    }

    /// Pick a uniformly random legal move, or `None` if no moves exist.
    pub fn random_move(&self) -> Option<Move> {
        let moves = movegen::generate_legal_moves(&self.inner);
        if moves.is_empty() { return None; }
        use rand::Rng;
        let idx = rand::thread_rng().gen_range(0..moves.len());
        Some(Move { inner: moves.into_iter().nth(idx).unwrap() })
    }

    /// Run an alpha-beta search.
    ///
    /// - `depth`: search depth in plies
    /// - `time_limit_ms`: wall-clock time limit in milliseconds (0 = unlimited)
    pub fn search(&mut self, depth: u32, time_limit_ms: u64) -> SearchResult {
        let r = search::search(&mut self.inner, depth, time_limit_ms);
        SearchResult {
            best_move: r.best_move.map(|m| Move { inner: m }),
            score: r.score,
            nodes: r.nodes,
            time_ms: r.time_ms,
        }
    }

    /// Render the board as a text string.
    pub fn display(&self) -> String {
        self.inner.display()
    }

    /// Iterate over all pieces of a given color.
    pub fn pieces(&self, color: Color) -> Vec<(Square, Piece)> {
        let c = color.raw() as usize;
        let mut result = Vec::new();
        for i in 0..self.inner.piece_list_len[c] {
            let sq = self.inner.piece_list[c][i];
            if sq == types::INVALID_SQ { continue; }
            let cell = self.inner.cells[sq as usize];
            if cell == types::EMPTY_CELL { continue; }
            result.push((
                Square::from_index(sq as usize),
                Piece {
                    type_id: types::cell_piece(cell),
                    color,
                },
            ));
        }
        result
    }
}

impl Clone for Board {
    fn clone(&self) -> Self {
        Board { inner: self.inner.clone() }
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.display())
    }
}

impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("Board")
            .field("side_to_move", &self.side_to_move())
            .field("move_number", &self.move_number())
            .field("black_pieces", &self.piece_count(Color::Black))
            .field("white_pieces", &self.piece_count(Color::White))
            .finish()
    }
}

// ============================================================
// PyO3 module entry point (only with "python" feature)
// ============================================================

#[cfg(feature = "python")]
use pyo3::prelude::*;

#[cfg(feature = "python")]
#[pymodule]
fn taikyokushogi(m: &Bound<'_, PyModule>) -> PyResult<()> {
    python::register(m)
}
