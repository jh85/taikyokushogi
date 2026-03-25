"""Static evaluation for Taikyoku Shogi."""

from .pieces import (
    BOARD_SIZE, BLACK, WHITE, PIECE_VALUE, MOVEMENTS, ROYAL_PIECES,
)
from .board import TaikyokuBoard

# Large value representing a won/lost position
MATE_SCORE = 1_000_000


def evaluate(board: TaikyokuBoard) -> int:
    """Evaluate the position from the current side to move's perspective.

    Returns positive if the position favors the side to move.
    """
    result = board.get_game_result()
    if result == 'black_wins':
        return MATE_SCORE if board.side_to_move == BLACK else -MATE_SCORE
    if result == 'white_wins':
        return MATE_SCORE if board.side_to_move == WHITE else -MATE_SCORE
    if result == 'draw':
        return 0

    score = 0

    # Material evaluation
    for color in (BLACK, WHITE):
        sign = 1 if color == board.side_to_move else -1
        for (r, c), piece in board.piece_positions[color].items():
            val = PIECE_VALUE.get(piece, 1000)
            score += sign * val

    # Simple mobility bonus (count of slide directions)
    for color in (BLACK, WHITE):
        sign = 1 if color == board.side_to_move else -1
        mobility = 0
        for (r, c), piece in board.piece_positions[color].items():
            m = MOVEMENTS.get(piece)
            if m:
                mobility += len(m.get('slides', []))
                mobility += len(m.get('jumps', []))
        score += sign * mobility * 10

    return score
