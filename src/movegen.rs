use crate::types::*;
use crate::pieces;
use crate::board::Board;

pub fn generate_legal_moves(board: &Board) -> Vec<Move> {
    let color = board.side_to_move;
    let c = color as usize;
    let rt = ray_table();
    let mut moves = Vec::with_capacity(512);

    for i in 0..board.piece_list_len[c] {
        let sq = board.piece_list[c][i] as usize;
        if sq == INVALID_SQ as usize { continue; }
        let cell = board.cells[sq];
        if cell == EMPTY_CELL { continue; }
        let pt = cell_piece(cell);
        let mv = pieces::movement(pt);

        gen_slides(board, sq, pt, color, mv, rt, &mut moves);
        gen_jumps(board, sq, pt, color, mv, &mut moves);

        if mv.hook.is_some() {
            gen_hooks(board, sq, pt, color, mv, rt, &mut moves);
        }
        if mv.area > 0 {
            gen_area(board, sq, pt, color, mv, &mut moves);
        }
        if !mv.range_capture.is_empty() {
            gen_range_capture(board, sq, pt, color, mv, rt, &mut moves);
        }
        if mv.igui {
            gen_igui(board, sq, pt, color, &mut moves);
        }
    }
    moves
}

#[inline]
fn can_promote(pt: u16) -> bool {
    pieces::promotes_to(pt).is_some()
}

/// Zone-based promotion rules:
/// 1. Entering the promotion zone from outside -> may promote
/// 2. Capturing inside the promotion zone -> may promote
/// 3. Reaching the farthest rank with a forward-only piece -> MUST promote
/// 4. Promotion is optional otherwise
fn add_move(moves: &mut Vec<Move>, from: u16, to: u16, pt: u16, color: u8, target: Cell) {
    let captured = if target != EMPTY_CELL { cell_piece(target) } else { 0 };
    let cap_color = if target != EMPTY_CELL { cell_color(target) } else { 0 };

    if !can_promote(pt) {
        // No promotion possible — just add the plain move
        moves.push(Move {
            from_sq: from, to_sq: to, promotion: false,
            captured_piece: captured, captured_color: cap_color,
            is_igui: false, mid_sq: INVALID_SQ, mid_piece: 0, mid_color: 0,
            range_caps: None,
        });
        return;
    }

    let from_in_zone = in_promo_zone(from as usize, color);
    let to_in_zone = in_promo_zone(to as usize, color);
    let is_capture = captured != 0;

    // Determine if promotion is allowed this move
    let may_promote =
        (!from_in_zone && to_in_zone) ||       // entering zone
        (from_in_zone && to_in_zone && is_capture); // capture inside zone

    // Determine if promotion is forced (forward-only piece at farthest rank)
    let must_promote = to_in_zone
        && is_farthest_rank(to as usize, color)
        && pieces::must_promote_at_far_rank(pt);

    if must_promote {
        // Only the promoted move
        moves.push(Move {
            from_sq: from, to_sq: to, promotion: true,
            captured_piece: captured, captured_color: cap_color,
            is_igui: false, mid_sq: INVALID_SQ, mid_piece: 0, mid_color: 0,
            range_caps: None,
        });
    } else if may_promote {
        // Both options: promote or don't
        moves.push(Move {
            from_sq: from, to_sq: to, promotion: false,
            captured_piece: captured, captured_color: cap_color,
            is_igui: false, mid_sq: INVALID_SQ, mid_piece: 0, mid_color: 0,
            range_caps: None,
        });
        moves.push(Move {
            from_sq: from, to_sq: to, promotion: true,
            captured_piece: captured, captured_color: cap_color,
            is_igui: false, mid_sq: INVALID_SQ, mid_piece: 0, mid_color: 0,
            range_caps: None,
        });
    } else {
        // No promotion available
        moves.push(Move {
            from_sq: from, to_sq: to, promotion: false,
            captured_piece: captured, captured_color: cap_color,
            is_igui: false, mid_sq: INVALID_SQ, mid_piece: 0, mid_color: 0,
            range_caps: None,
        });
    }
}

fn gen_slides(board: &Board, sq: usize, pt: u16, color: u8, mv: &Movement,
              rt: &RayTable, moves: &mut Vec<Move>) {
    for &(dir, max_range) in &mv.slides {
        let ray = rt.ray_for_color(sq, dir as usize, color);
        let limit = if max_range == 0 { ray.len() } else { (max_range as usize).min(ray.len()) };

        for j in 0..limit {
            let target_sq = ray[j] as usize;
            let target = board.cells[target_sq];
            if target == EMPTY_CELL {
                add_move(moves, sq as u16, target_sq as u16, pt, color, EMPTY_CELL);
            } else if cell_color(target) != color {
                add_move(moves, sq as u16, target_sq as u16, pt, color, target);
                break;
            } else {
                break; // friendly piece blocks
            }
        }
    }
}

fn gen_jumps(board: &Board, sq: usize, pt: u16, color: u8, mv: &Movement,
             moves: &mut Vec<Move>) {
    let r = sq_row(sq) as i32;
    let c = sq_col(sq) as i32;
    for &(jdr, jdc) in &mv.jumps {
        let (dr, dc) = if color == BLACK {
            (jdr as i32, jdc as i32)
        } else {
            (-(jdr as i32), -(jdc as i32))
        };
        let nr = r + dr;
        let nc = c + dc;
        if nr < 0 || nr >= BOARD_SIZE as i32 || nc < 0 || nc >= BOARD_SIZE as i32 { continue; }
        let nsq = nr as usize * BOARD_SIZE + nc as usize;
        let target = board.cells[nsq];
        if target == EMPTY_CELL {
            add_move(moves, sq as u16, nsq as u16, pt, color, EMPTY_CELL);
        } else if cell_color(target) != color {
            add_move(moves, sq as u16, nsq as u16, pt, color, target);
        }
    }
}

fn gen_hooks(board: &Board, sq: usize, pt: u16, color: u8, mv: &Movement,
             rt: &RayTable, moves: &mut Vec<Move>) {
    let dirs: &[usize] = match mv.hook {
        Some(HookType::Orthogonal) => &[N, E, S, W],
        Some(HookType::Diagonal) => &[NE, SE, SW, NW],
        None => return,
    };

    for &d in dirs {
        let ray = rt.ray_for_color(sq, d, color);
        for (_j, &mid_sq) in ray.iter().enumerate() {
            let mid = mid_sq as usize;
            let target = board.cells[mid];
            if target != EMPTY_CELL {
                if cell_color(target) != color {
                    add_move(moves, sq as u16, mid_sq, pt, color, target);
                }
                break;
            }
            // At this empty square, try turning 90 degrees
            let turn_dirs = match mv.hook {
                Some(HookType::Orthogonal) => {
                    if d == N || d == S { vec![E, W] } else { vec![N, S] }
                }
                Some(HookType::Diagonal) => {
                    match d {
                        NE => vec![NW, SE], SE => vec![NE, SW],
                        SW => vec![SE, NW], NW => vec![NE, SW],
                        _ => vec![],
                    }
                }
                None => vec![],
            };
            for td in turn_dirs {
                let turn_ray = rt.ray_for_color(mid, td, color);
                for &tsq in turn_ray {
                    let t = board.cells[tsq as usize];
                    if t == EMPTY_CELL {
                        add_move(moves, sq as u16, tsq, pt, color, EMPTY_CELL);
                    } else if cell_color(t) != color {
                        add_move(moves, sq as u16, tsq, pt, color, t);
                        break;
                    } else {
                        break;
                    }
                }
            }
        }
    }
}

fn gen_area(board: &Board, sq: usize, pt: u16, color: u8, mv: &Movement,
            moves: &mut Vec<Move>) {
    let r = sq_row(sq) as i32;
    let c = sq_col(sq) as i32;

    for d1 in 0..NUM_DIRS {
        let (dr1, dc1) = get_deltas(d1, color);
        let r1 = r + dr1;
        let c1 = c + dc1;
        if r1 < 0 || r1 >= BOARD_SIZE as i32 || c1 < 0 || c1 >= BOARD_SIZE as i32 { continue; }
        let sq1 = r1 as usize * BOARD_SIZE + c1 as usize;
        let t1 = board.cells[sq1];
        if t1 != EMPTY_CELL && cell_color(t1) == color { continue; }

        // First step destination
        add_move(moves, sq as u16, sq1 as u16, pt, color, t1);

        // Second step
        if mv.area >= 2 {
            for d2 in 0..NUM_DIRS {
                let (dr2, dc2) = get_deltas(d2, color);
                let r2 = r1 + dr2;
                let c2 = c1 + dc2;
                if r2 < 0 || r2 >= BOARD_SIZE as i32 || c2 < 0 || c2 >= BOARD_SIZE as i32 { continue; }
                let sq2 = r2 as usize * BOARD_SIZE + c2 as usize;
                if sq2 == sq { continue; } // back to start
                let t2 = board.cells[sq2];
                if t2 != EMPTY_CELL && cell_color(t2) == color { continue; }

                if t1 != EMPTY_CELL && cell_color(t1) != color {
                    // Captured at midpoint
                    let mut m = Move::simple(sq as u16, sq2 as u16);
                    m.mid_sq = sq1 as u16;
                    m.mid_piece = cell_piece(t1);
                    m.mid_color = cell_color(t1);
                    if t2 != EMPTY_CELL {
                        m.captured_piece = cell_piece(t2);
                        m.captured_color = cell_color(t2);
                    }
                    moves.push(m);
                } else {
                    add_move(moves, sq as u16, sq2 as u16, pt, color, t2);
                }
            }
        }
    }
}

fn gen_range_capture(board: &Board, sq: usize, pt: u16, color: u8, mv: &Movement,
                     rt: &RayTable, moves: &mut Vec<Move>) {
    let piece_rank = pieces::rank(pt);

    for &dir in &mv.range_capture {
        let ray = rt.ray_for_color(sq, dir as usize, color);
        let mut captured_list: Vec<(u16, u16, u8)> = Vec::new();

        for &rsq in ray {
            let target = board.cells[rsq as usize];
            if target == EMPTY_CELL {
                let mut m = Move::simple(sq as u16, rsq);
                if !captured_list.is_empty() {
                    m.range_caps = Some(captured_list.clone());
                }
                moves.push(m);
            } else {
                let t_pt = cell_piece(target);
                let t_rank = pieces::rank(t_pt);
                if t_rank > piece_rank {
                    // Lower rank: capture and continue
                    captured_list.push((rsq, t_pt, cell_color(target)));
                    // Apply zone-based promotion logic for range capture
                    let from_in = in_promo_zone(sq, color);
                    let to_in = in_promo_zone(rsq as usize, color);
                    let may_promo = can_promote(pt) && (
                        (!from_in && to_in) || (from_in && to_in)
                    );
                    let must_promo = may_promo && is_farthest_rank(rsq as usize, color)
                        && pieces::must_promote_at_far_rank(pt);
                    if must_promo {
                        let mut m = Move::simple(sq as u16, rsq);
                        m.captured_piece = t_pt;
                        m.captured_color = cell_color(target);
                        m.range_caps = Some(captured_list.clone());
                        m.promotion = true;
                        moves.push(m);
                    } else if may_promo {
                        // Both options
                        let mut m1 = Move::simple(sq as u16, rsq);
                        m1.captured_piece = t_pt;
                        m1.captured_color = cell_color(target);
                        m1.range_caps = Some(captured_list.clone());
                        moves.push(m1);
                        let mut m2 = Move::simple(sq as u16, rsq);
                        m2.captured_piece = t_pt;
                        m2.captured_color = cell_color(target);
                        m2.range_caps = Some(captured_list.clone());
                        m2.promotion = true;
                        moves.push(m2);
                    } else {
                        let mut m = Move::simple(sq as u16, rsq);
                        m.captured_piece = t_pt;
                        m.captured_color = cell_color(target);
                        m.range_caps = Some(captured_list.clone());
                        moves.push(m);
                    }
                } else {
                    break; // blocked by equal or higher rank
                }
            }
        }
    }
}

fn gen_igui(board: &Board, sq: usize, pt: u16, color: u8, moves: &mut Vec<Move>) {
    for d in 0..NUM_DIRS {
        if let Some(nsq) = step_sq(sq, d, color) {
            let target = board.cells[nsq];
            if target != EMPTY_CELL && cell_color(target) != color {
                // Igui is a capture in place — zone promotion applies if piece is in zone
                let in_zone = in_promo_zone(sq, color);
                let may_promo = can_promote(pt) && in_zone;
                if may_promo {
                    // Both options (never forced since piece doesn't move to farthest rank)
                    let mut m1 = Move::simple(sq as u16, sq as u16);
                    m1.captured_piece = cell_piece(target);
                    m1.captured_color = cell_color(target);
                    m1.is_igui = true;
                    moves.push(m1);
                    let mut m2 = Move::simple(sq as u16, sq as u16);
                    m2.captured_piece = cell_piece(target);
                    m2.captured_color = cell_color(target);
                    m2.is_igui = true;
                    m2.promotion = true;
                    moves.push(m2);
                } else {
                    let mut m = Move::simple(sq as u16, sq as u16);
                    m.captured_piece = cell_piece(target);
                    m.captured_color = cell_color(target);
                    m.is_igui = true;
                    moves.push(m);
                }
            }
        }
    }
}
