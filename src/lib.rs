use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
use std::collections::HashMap;

mod types;
mod pieces;
mod board;
mod movegen;
mod eval;
mod search;

use types::*;
use board::Board;

// ============================================================
// PyO3 Module
// ============================================================

#[pyclass]
#[derive(Clone)]
struct PyBoard {
    inner: Board,
}

#[pymethods]
impl PyBoard {
    #[new]
    fn new() -> Self {
        PyBoard { inner: Board::new() }
    }

    fn setup_initial(&mut self) {
        self.inner.setup_initial();
    }

    fn at(&self, row: usize, col: usize) -> Option<(String, u8)> {
        let sq = sq_index(row, col);
        let cell = self.inner.cells[sq];
        if cell == EMPTY_CELL {
            None
        } else {
            let pt = cell_piece(cell);
            let color = cell_color(cell);
            Some((pieces::abbrev(pt).to_string(), color))
        }
    }

    #[getter]
    fn side_to_move(&self) -> u8 {
        self.inner.side_to_move
    }

    #[getter]
    fn move_number(&self) -> u32 {
        self.inner.move_number
    }

    fn black_piece_count(&self) -> usize {
        self.inner.piece_count[BLACK as usize]
    }

    fn white_piece_count(&self) -> usize {
        self.inner.piece_count[WHITE as usize]
    }

    fn game_result(&self) -> Option<String> {
        match self.inner.game_result() {
            Some(GameResult::BlackWins) => Some("black_wins".into()),
            Some(GameResult::WhiteWins) => Some("white_wins".into()),
            Some(GameResult::Draw) => Some("draw".into()),
            None => None,
        }
    }

    fn piece_positions(&self, py: Python, color: u8) -> PyResult<PyObject> {
        let dict = PyDict::new(py);
        let c = color as usize;
        for i in 0..self.inner.piece_list_len[c] {
            let sq = self.inner.piece_list[c][i];
            if sq == INVALID_SQ { continue; }
            let cell = self.inner.cells[sq as usize];
            if cell == EMPTY_CELL { continue; }
            let pt = cell_piece(cell);
            let r = sq as usize / BOARD_SIZE;
            let col = sq as usize % BOARD_SIZE;
            let key = PyTuple::new(py, &[r, col]).expect("failed to create tuple");
            dict.set_item(key, pieces::abbrev(pt))?;
        }
        Ok(dict.into())
    }

    fn royal_positions(&self, py: Python, color: u8) -> PyResult<PyObject> {
        let list = PyList::empty(py);
        let c = color as usize;
        for i in 0..self.inner.royal_count[c] {
            let sq = self.inner.royal_list[c][i];
            let r = sq as usize / BOARD_SIZE;
            let col = sq as usize % BOARD_SIZE;
            list.append(PyTuple::new(py, &[r, col]).expect("failed"))?;
        }
        Ok(list.into())
    }

    fn score(&self) -> i32 {
        eval::material_score(&self.inner)
    }

    fn apply_move_py(&mut self, from_r: usize, from_c: usize,
                     to_r: usize, to_c: usize, promotion: bool) -> bool {
        let from_sq = sq_index(from_r, from_c) as u16;
        let to_sq = sq_index(to_r, to_c) as u16;
        let moves = movegen::generate_legal_moves(&self.inner);
        for m in &moves {
            if m.from_sq == from_sq && m.to_sq == to_sq && m.promotion == promotion {
                self.inner.apply_move(m);
                return true;
            }
        }
        false
    }

    fn undo(&mut self) -> bool {
        self.inner.undo_move()
    }

    fn legal_moves_py(&self, py: Python) -> PyResult<PyObject> {
        let moves = movegen::generate_legal_moves(&self.inner);
        let list = PyList::empty(py);
        for m in &moves {
            let d = PyDict::new(py);
            let fr = m.from_sq as usize / BOARD_SIZE;
            let fc = m.from_sq as usize % BOARD_SIZE;
            let tr = m.to_sq as usize / BOARD_SIZE;
            let tc = m.to_sq as usize % BOARD_SIZE;
            d.set_item("from", vec![fr, fc])?;
            d.set_item("to", vec![tr, tc])?;
            d.set_item("promotion", m.promotion)?;
            d.set_item("is_igui", m.is_igui)?;
            if m.captured_piece != 0 {
                d.set_item("captured", pieces::abbrev(m.captured_piece))?;
            } else {
                d.set_item("captured", py.None())?;
            }
            list.append(d)?;
        }
        Ok(list.into())
    }

    fn moves_from_py(&self, py: Python, r: usize, c: usize) -> PyResult<PyObject> {
        let sq = sq_index(r, c) as u16;
        let moves = movegen::generate_legal_moves(&self.inner);
        let list = PyList::empty(py);
        let mut seen = std::collections::HashSet::new();
        for m in &moves {
            if m.from_sq == sq {
                let key = (m.to_sq, m.promotion, m.is_igui);
                if seen.insert(key) {
                    let d = PyDict::new(py);
                    let tr = m.to_sq as usize / BOARD_SIZE;
                    let tc = m.to_sq as usize % BOARD_SIZE;
                    d.set_item("to", vec![tr, tc])?;
                    d.set_item("promotion", m.promotion)?;
                    d.set_item("is_igui", m.is_igui)?;
                    if m.captured_piece != 0 {
                        d.set_item("captured", pieces::name(m.captured_piece))?;
                    } else {
                        d.set_item("captured", py.None())?;
                    }
                    list.append(d)?;
                }
            }
        }
        Ok(list.into())
    }

    fn random_move_py(&self) -> Option<(usize, usize, usize, usize, bool)> {
        let moves = movegen::generate_legal_moves(&self.inner);
        if moves.is_empty() { return None; }
        use rand::Rng;
        let idx = rand::thread_rng().gen_range(0..moves.len());
        let m = &moves[idx];
        let fr = m.from_sq as usize / BOARD_SIZE;
        let fc = m.from_sq as usize % BOARD_SIZE;
        let tr = m.to_sq as usize / BOARD_SIZE;
        let tc = m.to_sq as usize % BOARD_SIZE;
        Some((fr, fc, tr, tc, m.promotion))
    }

    fn search_py(&mut self, depth: u32, time_limit_ms: u64) -> PyResult<(Option<(usize, usize, usize, usize, bool)>, i32, u64, u64)> {
        let result = search::search(&mut self.inner, depth, time_limit_ms);
        let mv = result.best_move.map(|m| {
            let fr = m.from_sq as usize / BOARD_SIZE;
            let fc = m.from_sq as usize % BOARD_SIZE;
            let tr = m.to_sq as usize / BOARD_SIZE;
            let tc = m.to_sq as usize % BOARD_SIZE;
            (fr, fc, tr, tc, m.promotion)
        });
        Ok((mv, result.score, result.nodes, result.time_ms))
    }

    fn display(&self) -> String {
        self.inner.display()
    }
}

#[pyfunction]
fn piece_name(abbrev: &str) -> String {
    if let Some(pt) = pieces::find_by_abbrev(abbrev) {
        pieces::name(pt).to_string()
    } else {
        abbrev.to_string()
    }
}

#[pyfunction]
fn piece_value(abbrev: &str) -> i32 {
    if let Some(pt) = pieces::find_by_abbrev(abbrev) {
        pieces::value(pt)
    } else {
        1000
    }
}

#[pyfunction]
fn piece_promotes_to(abbrev: &str) -> Option<String> {
    if let Some(pt) = pieces::find_by_abbrev(abbrev) {
        pieces::promotes_to(pt).map(|p| pieces::abbrev(p).to_string())
    } else {
        None
    }
}

#[pyfunction]
fn piece_info(py: Python, abbrev: &str) -> PyResult<PyObject> {
    let d = PyDict::new(py);
    d.set_item("abbrev", abbrev)?;
    if let Some(pt) = pieces::find_by_abbrev(abbrev) {
        d.set_item("name", pieces::name(pt))?;
        d.set_item("value", pieces::value(pt))?;
        let promo = pieces::promotes_to(pt).map(|p| pieces::name(p).to_string());
        d.set_item("promotes_to", promo)?;
        let mv = pieces::movement(pt);
        d.set_item("slide_directions", mv.slides.len())?;
        d.set_item("jump_destinations", mv.jumps.len())?;
        let mut specials = Vec::new();
        if mv.hook.is_some() { specials.push("hook"); }
        if mv.area > 0 { specials.push("area"); }
        if !mv.range_capture.is_empty() { specials.push("range capture"); }
        if mv.igui { specials.push("igui"); }
        d.set_item("specials", specials)?;
    } else {
        d.set_item("name", abbrev)?;
        d.set_item("value", 0)?;
        d.set_item("promotes_to", py.None())?;
        d.set_item("slide_directions", 0)?;
        d.set_item("jump_destinations", 0)?;
        let empty: Vec<&str> = vec![];
        d.set_item("specials", empty)?;
    }
    Ok(d.into())
}

#[pymodule]
fn taikyoku_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyBoard>()?;
    m.add_function(wrap_pyfunction!(piece_name, m)?)?;
    m.add_function(wrap_pyfunction!(piece_value, m)?)?;
    m.add_function(wrap_pyfunction!(piece_promotes_to, m)?)?;
    m.add_function(wrap_pyfunction!(piece_info, m)?)?;
    m.add("BOARD_SIZE", BOARD_SIZE)?;
    m.add("BLACK", BLACK)?;
    m.add("WHITE", WHITE)?;
    Ok(())
}
