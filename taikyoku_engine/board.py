"""Board representation for Taikyoku Shogi."""

from .pieces import (
    BOARD_SIZE, BLACK, WHITE, MOVEMENTS, PROMOTES_TO,
    ROYAL_PIECES, PIECE_RANK, RANK_ROYAL, get_initial_board,
    PIECE_NAME, PIECE_VALUE,
)
from .move import Move


class TaikyokuBoard:
    """36x36 board for Taikyoku Shogi.

    Board is stored as board[row][col] where:
      - Row 0 = top (White's back rank)
      - Row 35 = bottom (Black's back rank)
      - Each cell is (piece_abbrev, color) or None
    """

    def __init__(self):
        self.board = [[None] * BOARD_SIZE for _ in range(BOARD_SIZE)]
        self.side_to_move = BLACK
        self.move_number = 1
        self.move_history = []  # Stack of (move, undo_info) for undo
        # Track royal pieces for each side
        self.royals = {BLACK: set(), WHITE: set()}
        # Track all piece positions for each side
        self.piece_positions = {BLACK: {}, WHITE: {}}  # color -> {(r,c): abbrev}

    def setup_initial(self):
        """Set up the initial position."""
        self.board = get_initial_board()
        self.side_to_move = BLACK
        self.move_number = 1
        self.move_history = []
        self._rebuild_indices()

    def _rebuild_indices(self):
        """Rebuild piece position indices from board state."""
        self.royals = {BLACK: set(), WHITE: set()}
        self.piece_positions = {BLACK: {}, WHITE: {}}
        for r in range(BOARD_SIZE):
            for c in range(BOARD_SIZE):
                cell = self.board[r][c]
                if cell is not None:
                    piece, color = cell
                    self.piece_positions[color][(r, c)] = piece
                    if piece in ROYAL_PIECES:
                        self.royals[color].add((r, c))

    def at(self, row, col):
        """Get piece at (row, col). Returns (abbrev, color) or None."""
        return self.board[row][col]

    def set_piece(self, row, col, piece, color):
        """Place a piece on the board."""
        old = self.board[row][col]
        if old is not None:
            old_piece, old_color = old
            self.piece_positions[old_color].pop((row, col), None)
            if old_piece in ROYAL_PIECES:
                self.royals[old_color].discard((row, col))

        self.board[row][col] = (piece, color)
        self.piece_positions[color][(row, col)] = piece
        if piece in ROYAL_PIECES:
            self.royals[color].add((row, col))

    def remove_piece(self, row, col):
        """Remove piece from (row, col)."""
        cell = self.board[row][col]
        if cell is not None:
            piece, color = cell
            self.piece_positions[color].pop((row, col), None)
            if piece in ROYAL_PIECES:
                self.royals[color].discard((row, col))
        self.board[row][col] = None

    def apply_move(self, move: Move):
        """Apply a move to the board. Returns undo info."""
        fr, fc = move.from_sq
        tr, tc = move.to_sq
        piece, color = self.board[fr][fc]

        # Save undo info
        undo = {
            'from_cell': self.board[fr][fc],
            'to_cell': self.board[tr][tc],
            'side': self.side_to_move,
            'move_number': self.move_number,
        }

        # Handle range captures (capture all pieces along path)
        if move.range_captures:
            undo['range_captures'] = []
            for rr, rc, rp, rcol in move.range_captures:
                undo['range_captures'].append((rr, rc, self.board[rr][rc]))
                self.remove_piece(rr, rc)

        # Handle lion mid-capture
        if move.mid_sq is not None:
            mr, mc = move.mid_sq
            undo['mid_cell'] = self.board[mr][mc]
            self.remove_piece(mr, mc)

        # Handle igui (capture without moving)
        if move.is_igui:
            captured_cell = self.board[tr][tc]
            undo['to_cell'] = captured_cell
            self.remove_piece(tr, tc)
            # Piece stays in place but may promote
            if move.promotion and piece in PROMOTES_TO:
                new_piece = PROMOTES_TO[piece]
                self.remove_piece(fr, fc)
                self.set_piece(fr, fc, new_piece, color)
            self.side_to_move = 1 - self.side_to_move
            if self.side_to_move == BLACK:
                self.move_number += 1
            self.move_history.append((move, undo))
            return undo

        # Standard move
        # Remove piece from origin
        self.remove_piece(fr, fc)

        # Capture at destination
        if self.board[tr][tc] is not None:
            self.remove_piece(tr, tc)

        # Determine if piece promotes
        final_piece = piece
        if move.promotion and piece in PROMOTES_TO:
            final_piece = PROMOTES_TO[piece]

        # Place piece at destination
        self.set_piece(tr, tc, final_piece, color)

        # Update side to move
        self.side_to_move = 1 - self.side_to_move
        if self.side_to_move == BLACK:
            self.move_number += 1

        self.move_history.append((move, undo))
        return undo

    def undo_move(self):
        """Undo the last move."""
        if not self.move_history:
            return

        move, undo = self.move_history.pop()
        fr, fc = move.from_sq
        tr, tc = move.to_sq

        # Restore side to move and move number
        self.side_to_move = undo['side']
        self.move_number = undo['move_number']

        if move.is_igui:
            # Restore the piece at from (may have promoted)
            self.remove_piece(fr, fc)
            from_piece, from_color = undo['from_cell']
            self.set_piece(fr, fc, from_piece, from_color)
            # Restore captured piece at to
            if undo['to_cell'] is not None:
                tp, tc_col = undo['to_cell']
                self.set_piece(tr, tc, tp, tc_col)
            return

        # Remove the moved piece from destination
        self.remove_piece(tr, tc)

        # Restore origin
        if undo['from_cell'] is not None:
            fp, fcol = undo['from_cell']
            self.set_piece(fr, fc, fp, fcol)

        # Restore destination (captured piece)
        if undo['to_cell'] is not None:
            tp, tcol = undo['to_cell']
            self.set_piece(tr, tc, tp, tcol)

        # Restore mid-capture
        if move.mid_sq is not None and 'mid_cell' in undo:
            mr, mc = move.mid_sq
            if undo['mid_cell'] is not None:
                mp, mcol = undo['mid_cell']
                self.set_piece(mr, mc, mp, mcol)

        # Restore range captures
        if 'range_captures' in undo:
            for rr, rc, cell in undo['range_captures']:
                if cell is not None:
                    rp, rcol = cell
                    self.set_piece(rr, rc, rp, rcol)

    def is_valid(self, row, col):
        """Check if (row, col) is within the board."""
        return 0 <= row < BOARD_SIZE and 0 <= col < BOARD_SIZE

    def has_royals(self, color):
        """Check if the given side still has royal pieces."""
        return len(self.royals[color]) > 0

    def get_game_result(self):
        """Check game result.

        Returns:
          'black_wins' if White has no royals
          'white_wins' if Black has no royals
          'draw' if neither has royals (unlikely)
          None if game is ongoing
        """
        black_has = self.has_royals(BLACK)
        white_has = self.has_royals(WHITE)
        if not white_has and not black_has:
            return 'draw'
        if not white_has:
            return 'black_wins'
        if not black_has:
            return 'white_wins'
        return None

    def display(self, compact=True):
        """Return a string representation of the board."""
        lines = []
        for r in range(BOARD_SIZE):
            row_str = []
            for c in range(BOARD_SIZE):
                cell = self.board[r][c]
                if cell is None:
                    row_str.append('.. ')
                else:
                    piece, color = cell
                    prefix = 'v' if color == WHITE else '^'
                    abbrev = piece[:2].ljust(2)
                    row_str.append(f"{prefix}{abbrev}")
            lines.append(f"{36-r:2d} {''.join(row_str)}")
        return '\n'.join(lines)

    def copy(self):
        """Create a deep copy of the board."""
        new = TaikyokuBoard()
        new.board = [row[:] for row in self.board]
        new.side_to_move = self.side_to_move
        new.move_number = self.move_number
        new._rebuild_indices()
        return new
