"""TSFEN notation for Taikyoku Shogi positions.

Format: rank/rank/.../rank side move_number

Each rank lists pieces left to right. Empty squares use numbers (like FEN).
Piece format: Color prefix (uppercase=Black, lowercase=White) + abbreviation.
Multi-character abbreviations are wrapped: e.g., (CP) for Crown Prince.

Example fragment: L(TS)(RR)W... for Black's back rank.
"""

from .pieces import BOARD_SIZE, BLACK, WHITE, PIECE_NAME
from .board import TaikyokuBoard


def to_tsfen(board: TaikyokuBoard) -> str:
    """Convert board position to TSFEN string."""
    ranks = []
    for r in range(BOARD_SIZE):
        rank_str = _encode_rank(board, r)
        ranks.append(rank_str)

    side = 'b' if board.side_to_move == BLACK else 'w'
    return f"{'/'.join(ranks)} {side} {board.move_number}"


def from_tsfen(tsfen: str) -> TaikyokuBoard:
    """Parse TSFEN string into a TaikyokuBoard."""
    parts = tsfen.strip().split()
    rank_str = parts[0]
    side = parts[1] if len(parts) > 1 else 'b'
    move_num = int(parts[2]) if len(parts) > 2 else 1

    board = TaikyokuBoard()
    board.side_to_move = BLACK if side == 'b' else WHITE
    board.move_number = move_num

    ranks = rank_str.split('/')
    if len(ranks) != BOARD_SIZE:
        raise ValueError(f"Expected {BOARD_SIZE} ranks, got {len(ranks)}")

    for r, rank in enumerate(ranks):
        _decode_rank(board, r, rank)

    board._rebuild_indices()
    return board


def _encode_rank(board, row):
    """Encode a single rank as TSFEN."""
    parts = []
    empty_count = 0

    for c in range(BOARD_SIZE):
        cell = board.at(row, c)
        if cell is None:
            empty_count += 1
        else:
            if empty_count > 0:
                parts.append(str(empty_count))
                empty_count = 0
            piece, color = cell
            parts.append(_encode_piece(piece, color))

    if empty_count > 0:
        parts.append(str(empty_count))

    return ''.join(parts)


def _encode_piece(piece, color):
    """Encode a piece as TSFEN token.

    Black pieces: uppercase, White pieces: lowercase.
    Single-char abbreviations: just the char.
    Multi-char abbreviations: wrapped in parentheses.
    """
    if color == BLACK:
        s = piece.upper()
    else:
        s = piece.lower()

    if len(piece) == 1:
        return s
    else:
        return f"({s})"


def _decode_rank(board, row, rank_str):
    """Decode a TSFEN rank string and place pieces on the board."""
    col = 0
    i = 0
    n = len(rank_str)

    while i < n and col < BOARD_SIZE:
        ch = rank_str[i]

        if ch.isdigit():
            # Empty squares
            num_str = ''
            while i < n and rank_str[i].isdigit():
                num_str += rank_str[i]
                i += 1
            col += int(num_str)

        elif ch == '(':
            # Multi-character piece abbreviation
            i += 1  # skip '('
            abbrev = ''
            while i < n and rank_str[i] != ')':
                abbrev += rank_str[i]
                i += 1
            i += 1  # skip ')'

            # Determine color from case
            if abbrev[0].isupper() or (abbrev.startswith('+') and len(abbrev) > 1 and abbrev[1].isupper()):
                color = BLACK
                piece = abbrev.upper()
            else:
                color = WHITE
                piece = abbrev.upper()

            board.set_piece(row, col, piece, color)
            col += 1

        else:
            # Single character piece
            if ch.isupper():
                color = BLACK
                piece = ch
            else:
                color = WHITE
                piece = ch.upper()

            board.set_piece(row, col, piece, color)
            col += 1
            i += 1
