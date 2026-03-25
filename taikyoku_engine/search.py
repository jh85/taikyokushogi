"""Alpha-beta search for Taikyoku Shogi."""

import time
from .pieces import PIECE_VALUE, ROYAL_PIECES
from .board import TaikyokuBoard
from .movegen import generate_legal_moves
from .evaluation import evaluate, MATE_SCORE
from .move import Move


class SearchResult:
    def __init__(self, best_move=None, score=0, depth=0, nodes=0, time_ms=0):
        self.best_move = best_move
        self.score = score
        self.depth = depth
        self.nodes = nodes
        self.time_ms = time_ms


def search(board: TaikyokuBoard, depth: int = 2, time_limit_ms: int = 0) -> SearchResult:
    """Search for the best move using alpha-beta pruning.

    Args:
        board: Current board position
        depth: Search depth (plies)
        time_limit_ms: Time limit in milliseconds (0 = no limit)

    Returns:
        SearchResult with best move, score, and stats
    """
    searcher = _Searcher(board, time_limit_ms)
    result = SearchResult()
    start = time.time()

    try:
        score = searcher.alphabeta(depth, -MATE_SCORE - 1, MATE_SCORE + 1)
        result.best_move = searcher.best_move
        result.score = score
    except _TimeoutError:
        pass

    result.depth = depth
    result.nodes = searcher.nodes
    result.time_ms = int((time.time() - start) * 1000)
    return result


class _TimeoutError(Exception):
    pass


class _Searcher:
    def __init__(self, board, time_limit_ms):
        self.board = board
        self.best_move = None
        self.nodes = 0
        self.start_time = time.time()
        self.time_limit = time_limit_ms / 1000.0 if time_limit_ms > 0 else 0

    def _check_time(self):
        if self.time_limit > 0 and (time.time() - self.start_time) > self.time_limit:
            raise _TimeoutError()

    def alphabeta(self, depth, alpha, beta):
        """Alpha-beta search."""
        self.nodes += 1

        if self.nodes % 10000 == 0:
            self._check_time()

        # Terminal check
        result = self.board.get_game_result()
        if result is not None:
            if result == 'draw':
                return 0
            side = self.board.side_to_move
            if (result == 'black_wins' and side == 0) or \
               (result == 'white_wins' and side == 1):
                return MATE_SCORE
            return -MATE_SCORE

        if depth <= 0:
            return evaluate(self.board)

        moves = generate_legal_moves(self.board)
        if not moves:
            return evaluate(self.board)

        # Move ordering: captures first, then by captured piece value
        moves.sort(key=lambda m: _move_order_key(m), reverse=True)

        is_root = (depth == self._root_depth if hasattr(self, '_root_depth') else False)
        best_move = moves[0]
        best_score = -MATE_SCORE - 1

        for move in moves:
            self.board.apply_move(move)
            score = -self.alphabeta(depth - 1, -beta, -alpha)
            self.board.undo_move()

            if score > best_score:
                best_score = score
                best_move = move

            if score > alpha:
                alpha = score

            if alpha >= beta:
                break  # Beta cutoff

        # Save best move at root
        if not hasattr(self, '_root_depth'):
            self._root_depth = depth
        if depth == self._root_depth:
            self.best_move = best_move

        return best_score


def _move_order_key(move):
    """Score a move for move ordering. Higher = search first."""
    score = 0
    if move.captured:
        score += PIECE_VALUE.get(move.captured, 1000) * 10
    if move.promotion:
        score += 5000
    if move.range_captures:
        score += sum(PIECE_VALUE.get(p, 1000) for _, _, p, _ in move.range_captures) * 10
    if move.mid_captured:
        score += PIECE_VALUE.get(move.mid_captured, 1000) * 5
    return score
