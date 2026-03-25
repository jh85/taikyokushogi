"""Move generation for Taikyoku Shogi."""

from .pieces import (
    BOARD_SIZE, BLACK, WHITE, MOVEMENTS, PROMOTES_TO,
    ROYAL_PIECES, PIECE_RANK, RANK_ROYAL, RANK_GREAT, RANK_VICE,
    get_deltas, DELTAS, N, NE, E, SE, S, SW, W, NW, ALL_DIRS,
)
from .move import Move
from .board import TaikyokuBoard


def generate_pseudo_legal_moves(board: TaikyokuBoard):
    """Generate all pseudo-legal moves for the side to move.

    Does NOT filter out moves that leave own royals in check
    (Taikyoku Shogi doesn't have check rules - you can move into check,
     but losing all royals loses the game).
    """
    color = board.side_to_move
    moves = []

    for (r, c), piece in list(board.piece_positions[color].items()):
        movement = MOVEMENTS.get(piece)
        if movement is None:
            continue

        # Standard slides (step and range moves)
        _gen_slides(board, r, c, piece, color, movement, moves)

        # Jump moves
        _gen_jumps(board, r, c, piece, color, movement, moves)

        # Hook moves
        if movement.get('hook'):
            _gen_hooks(board, r, c, piece, color, movement, moves)

        # Area moves (lion-like)
        if movement.get('area', 0) > 0:
            _gen_area(board, r, c, piece, color, movement, moves)

        # Range capture
        if movement.get('range_capture'):
            _gen_range_capture(board, r, c, piece, color, movement, moves)

        # Igui
        if movement.get('igui'):
            _gen_igui(board, r, c, piece, color, moves)

    return moves


def generate_legal_moves(board: TaikyokuBoard):
    """Generate all legal moves. In Taikyoku Shogi, all pseudo-legal moves
    are legal since there's no check/checkmate rule.

    However, we should still filter out moves that are clearly suicidal
    (moving your last royal into capture) for better play.
    For now, return all pseudo-legal moves.
    """
    return generate_pseudo_legal_moves(board)


def _can_promote(piece):
    """Check if a piece can promote (has a promotion target)."""
    return piece in PROMOTES_TO


def _gen_slides(board, r, c, piece, color, movement, moves):
    """Generate slide/step moves along straight lines."""
    for direction, max_range in movement.get('slides', []):
        dr, dc = get_deltas(direction, color)
        if max_range == 0:
            max_range = BOARD_SIZE  # unlimited
        for dist in range(1, max_range + 1):
            nr, nc = r + dr * dist, c + dc * dist
            if not board.is_valid(nr, nc):
                break
            target = board.at(nr, nc)
            if target is None:
                # Empty square - can move here
                _add_move(moves, r, c, nr, nc, piece, color, None)
            elif target[1] != color:
                # Enemy piece - can capture
                _add_move(moves, r, c, nr, nc, piece, color, target)
                break  # Can't go further after capture
            else:
                # Friendly piece - blocked
                break


def _gen_jumps(board, r, c, piece, color, movement, moves):
    """Generate jump moves (bypassing intervening pieces)."""
    for jdr, jdc in movement.get('jumps', []):
        # Adjust for color
        if color == WHITE:
            jdr, jdc = -jdr, -jdc
        nr, nc = r + jdr, c + jdc
        if not board.is_valid(nr, nc):
            continue
        target = board.at(nr, nc)
        if target is None:
            _add_move(moves, r, c, nr, nc, piece, color, None)
        elif target[1] != color:
            _add_move(moves, r, c, nr, nc, piece, color, target)


def _gen_hooks(board, r, c, piece, color, movement, moves):
    """Generate hook moves (move along a line, then turn 90 degrees)."""
    hook_type = movement['hook']
    if hook_type == 'orth':
        # Hook on orthogonal lines: move along R, can turn 90 to continue on R
        dirs = [N, E, S, W]
    else:  # 'diag'
        dirs = [NE, SE, SW, NW]

    for d in dirs:
        dr, dc = get_deltas(d, color)
        # Move along primary direction
        for dist1 in range(1, BOARD_SIZE):
            nr, nc = r + dr * dist1, c + dc * dist1
            if not board.is_valid(nr, nc):
                break
            target = board.at(nr, nc)
            if target is not None:
                if target[1] != color:
                    # Can capture here (stop)
                    _add_move(moves, r, c, nr, nc, piece, color, target)
                break  # Blocked

            # Can stop here (already added by slides if applicable)
            # Now try turning 90 degrees at this point
            if hook_type == 'orth':
                turn_dirs = [E, W] if d in (N, S) else [N, S]
            else:
                turn_dirs_map = {
                    NE: [NW, SE], SE: [NE, SW],
                    SW: [SE, NW], NW: [NE, SW],
                }
                turn_dirs = turn_dirs_map.get(d, [])

            for td in turn_dirs:
                tdr, tdc = get_deltas(td, color)
                for dist2 in range(1, BOARD_SIZE):
                    nr2, nc2 = nr + tdr * dist2, nc + tdc * dist2
                    if not board.is_valid(nr2, nc2):
                        break
                    target2 = board.at(nr2, nc2)
                    if target2 is None:
                        _add_move(moves, r, c, nr2, nc2, piece, color, None)
                    elif target2[1] != color:
                        _add_move(moves, r, c, nr2, nc2, piece, color, target2)
                        break
                    else:
                        break


def _gen_area(board, r, c, piece, color, movement, moves):
    """Generate area moves (lion-like: multiple steps per turn)."""
    area_range = movement['area']
    # For area=2: can make up to 2 steps in any direction
    # This means it can reach any square within Manhattan distance 2
    # It can also capture one piece, then continue to an adjacent square

    # Simple implementation: generate all destination squares within range
    # For area=2: all squares reachable in 1 or 2 king steps
    visited = set()
    visited.add((r, c))

    # First step destinations
    for d in ALL_DIRS:
        dr, dc = get_deltas(d, color)
        r1, c1 = r + dr, c + dc
        if not board.is_valid(r1, c1):
            continue
        target1 = board.at(r1, c1)
        if target1 is not None and target1[1] == color:
            continue  # Blocked by friendly piece

        # Can reach (r1, c1) after first step
        if (r1, c1) not in visited:
            visited.add((r1, c1))
            if target1 is None:
                # Empty - can stop here or continue
                _add_move(moves, r, c, r1, c1, piece, color, None)
            else:
                # Enemy - capture and stop, or capture and continue
                _add_move(moves, r, c, r1, c1, piece, color, target1)

            # Second step (if area >= 2)
            if area_range >= 2:
                for d2 in ALL_DIRS:
                    dr2, dc2 = get_deltas(d2, color)
                    r2, c2 = r1 + dr2, c1 + dc2
                    if not board.is_valid(r2, c2):
                        continue
                    if (r2, c2) == (r, c):
                        # Returning to start - this is igui if captured on way
                        if target1 is not None and target1[1] != color:
                            m = Move(
                                from_sq=(r, c), to_sq=(r, c),
                                captured=target1[0], captured_color=target1[1],
                                mid_sq=(r1, c1), mid_captured=target1[0],
                                mid_captured_color=target1[1], is_igui=True
                            )
                            moves.append(m)
                        continue
                    target2 = board.at(r2, c2)
                    if target2 is not None and target2[1] == color:
                        continue
                    # Can reach (r2, c2) via (r1, c1)
                    if target1 is not None and target1[1] != color:
                        # Captured at mid-point
                        m = Move(
                            from_sq=(r, c), to_sq=(r2, c2),
                            mid_sq=(r1, c1), mid_captured=target1[0],
                            mid_captured_color=target1[1],
                        )
                        if target2 is not None:
                            m.captured = target2[0]
                            m.captured_color = target2[1]
                        moves.append(m)
                    elif target2 is not None and target2[1] != color:
                        # No capture at midpoint, capture at destination
                        _add_move(moves, r, c, r2, c2, piece, color, target2)
                    else:
                        # No captures along the way
                        _add_move(moves, r, c, r2, c2, piece, color, None)


def _gen_range_capture(board, r, c, piece, color, movement, moves):
    """Generate range capture moves (fly over pieces, capturing all).

    Range capturers fly over pieces of LOWER rank (higher numeric rank value),
    capturing all of them (friend or foe). They cannot fly over pieces of
    equal or higher rank (lower numeric value). They can also land on empty
    squares or on lower-rank pieces.
    """
    piece_rank = PIECE_RANK.get(piece, 4)

    for d in movement.get('range_capture', []):
        dr, dc = get_deltas(d, color)
        captured_list = []

        for dist in range(1, BOARD_SIZE):
            nr, nc = r + dr * dist, c + dc * dist
            if not board.is_valid(nr, nc):
                break

            target = board.at(nr, nc)
            if target is None:
                # Empty square - can land here
                m = Move(from_sq=(r, c), to_sq=(nr, nc),
                         range_captures=list(captured_list) if captured_list else None)
                moves.append(m)
            else:
                t_piece, t_color = target
                t_rank = PIECE_RANK.get(t_piece, 4)
                if t_rank > piece_rank:
                    # Lower rank piece - capture it and continue
                    captured_list.append((nr, nc, t_piece, t_color))
                    # Can also choose to land here
                    m = Move(from_sq=(r, c), to_sq=(nr, nc),
                             range_captures=list(captured_list))
                    moves.append(m)
                else:
                    # Equal or higher rank - blocked, cannot pass
                    break


def _gen_igui(board, r, c, piece, color, moves):
    """Generate igui moves (capture adjacent piece without moving)."""
    for d in ALL_DIRS:
        dr, dc = get_deltas(d, color)
        nr, nc = r + dr, c + dc
        if not board.is_valid(nr, nc):
            continue
        target = board.at(nr, nc)
        if target is not None and target[1] != color:
            m = Move(
                from_sq=(r, c), to_sq=(r, c),
                captured=target[0], captured_color=target[1],
                is_igui=True,
            )
            # Check for promotion on igui capture
            if _can_promote(piece):
                m2 = Move(
                    from_sq=(r, c), to_sq=(r, c),
                    captured=target[0], captured_color=target[1],
                    is_igui=True, promotion=True,
                )
                moves.append(m2)
            moves.append(m)


def _add_move(moves, fr, fc, tr, tc, piece, color, target):
    """Add a move, including promotion variant if applicable."""
    captured = target[0] if target else None
    cap_color = target[1] if target else None

    # Normal move
    m = Move(
        from_sq=(fr, fc), to_sq=(tr, tc),
        captured=captured, captured_color=cap_color,
    )
    moves.append(m)

    # Promotion variant: only when capturing (promotion by capture rule)
    if captured is not None and _can_promote(piece):
        m_promo = Move(
            from_sq=(fr, fc), to_sq=(tr, tc),
            promotion=True,
            captured=captured, captured_color=cap_color,
        )
        moves.append(m_promo)


def choose_random_move(board: TaikyokuBoard):
    """Choose a uniformly random legal move."""
    import random
    moves = generate_legal_moves(board)
    if not moves:
        return None
    return random.choice(moves)
