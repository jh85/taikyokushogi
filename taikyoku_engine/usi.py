"""USI (Universal Shogi Interface) protocol handler for Taikyoku Shogi."""

import sys
from .board import TaikyokuBoard
from .movegen import generate_legal_moves, choose_random_move
from .search import search
from .tsfen import to_tsfen, from_tsfen
from .move import Move, parse_sq_name


ENGINE_NAME = "TaikyokuEngine"
ENGINE_AUTHOR = "Taikyoku Shogi Project"


class USIHandler:
    def __init__(self):
        self.board = TaikyokuBoard()
        self.board.setup_initial()
        self.debug = False

    def run(self):
        """Main USI loop: read commands from stdin, write responses to stdout."""
        while True:
            try:
                line = input().strip()
            except EOFError:
                break

            if not line:
                continue

            parts = line.split()
            cmd = parts[0]

            if cmd == 'usi':
                self._cmd_usi()
            elif cmd == 'isready':
                self._cmd_isready()
            elif cmd == 'usinewgame':
                self._cmd_newgame()
            elif cmd == 'position':
                self._cmd_position(parts[1:])
            elif cmd == 'go':
                self._cmd_go(parts[1:])
            elif cmd == 'stop':
                pass  # Search stops are handled by time limits
            elif cmd == 'quit':
                break
            elif cmd == 'display' or cmd == 'd':
                self._cmd_display()
            elif cmd == 'moves':
                self._cmd_moves()
            elif cmd == 'tsfen':
                self._cmd_tsfen()
            elif cmd == 'perft':
                depth = int(parts[1]) if len(parts) > 1 else 1
                self._cmd_perft(depth)

    def _send(self, msg):
        print(msg, flush=True)

    def _cmd_usi(self):
        self._send(f"id name {ENGINE_NAME}")
        self._send(f"id author {ENGINE_AUTHOR}")
        self._send("option name Depth type spin default 1 min 1 max 10")
        self._send("usiok")

    def _cmd_isready(self):
        self._send("readyok")

    def _cmd_newgame(self):
        self.board = TaikyokuBoard()
        self.board.setup_initial()

    def _cmd_position(self, args):
        if not args:
            return

        if args[0] == 'startpos':
            self.board = TaikyokuBoard()
            self.board.setup_initial()
            moves_idx = 1
        elif args[0] == 'tsfen':
            # Find the 'moves' keyword to split TSFEN from moves
            moves_idx = len(args)
            for i, a in enumerate(args[1:], 1):
                if a == 'moves':
                    moves_idx = i
                    break
            tsfen_str = ' '.join(args[1:moves_idx])
            self.board = from_tsfen(tsfen_str)
        else:
            return

        # Apply moves
        if moves_idx < len(args) and args[moves_idx] == 'moves':
            for move_str in args[moves_idx + 1:]:
                move = _parse_move_str(move_str, self.board)
                if move:
                    self.board.apply_move(move)

    def _cmd_go(self, args):
        depth = 1
        time_limit = 0  # ms

        i = 0
        while i < len(args):
            if args[i] == 'depth' and i + 1 < len(args):
                depth = int(args[i + 1])
                i += 2
            elif args[i] == 'movetime' and i + 1 < len(args):
                time_limit = int(args[i + 1])
                i += 2
            elif args[i] == 'random':
                # Play random move
                move = choose_random_move(self.board)
                if move:
                    self._send(f"bestmove {move}")
                else:
                    self._send("bestmove resign")
                return
            else:
                i += 1

        result = search(self.board, depth=depth, time_limit_ms=time_limit)
        if result.best_move:
            info = f"info depth {result.depth} score cp {result.score} nodes {result.nodes} time {result.time_ms}"
            self._send(info)
            self._send(f"bestmove {result.best_move}")
        else:
            self._send("bestmove resign")

    def _cmd_display(self):
        self._send(self.board.display())
        side = "Black" if self.board.side_to_move == 0 else "White"
        self._send(f"Side to move: {side}")
        self._send(f"Move: {self.board.move_number}")
        b_count = len(self.board.piece_positions[0])
        w_count = len(self.board.piece_positions[1])
        self._send(f"Pieces: Black={b_count} White={w_count}")

    def _cmd_moves(self):
        moves = generate_legal_moves(self.board)
        self._send(f"Legal moves: {len(moves)}")
        for m in moves[:50]:  # Show first 50
            self._send(f"  {m}")
        if len(moves) > 50:
            self._send(f"  ... and {len(moves) - 50} more")

    def _cmd_tsfen(self):
        self._send(to_tsfen(self.board))

    def _cmd_perft(self, depth):
        count = _perft(self.board, depth)
        self._send(f"perft({depth}) = {count}")


def _perft(board, depth):
    """Count leaf nodes at given depth (for debugging move generation)."""
    if depth <= 0:
        return 1
    moves = generate_legal_moves(board)
    count = 0
    for move in moves:
        board.apply_move(move)
        count += _perft(board, depth - 1)
        board.undo_move()
    return count


def _parse_move_str(move_str, board):
    """Parse a move string like '18a17a' or '18a17a+' into a Move object."""
    promo = move_str.endswith('+')
    if promo:
        move_str = move_str[:-1]

    # Split into from and to squares
    # Find the split point - look for second digit sequence
    # Format: <file><rank><file><rank>
    # This is tricky because files are numbers and ranks are letters
    # Let's try to split by finding letter boundaries
    i = 0
    # Skip digits (from-file)
    while i < len(move_str) and move_str[i].isdigit():
        i += 1
    # Skip letters (from-rank)
    while i < len(move_str) and move_str[i].isalpha():
        i += 1

    from_str = move_str[:i]
    to_str = move_str[i:]

    try:
        fr, fc = parse_sq_name(from_str)
        tr, tc = parse_sq_name(to_str)
    except (ValueError, IndexError):
        return None

    # Find the matching legal move
    moves = generate_legal_moves(board)
    for m in moves:
        if m.from_sq == (fr, fc) and m.to_sq == (tr, tc) and m.promotion == promo:
            return m

    # Fallback: create a basic move
    target = board.at(tr, tc)
    return Move(
        from_sq=(fr, fc), to_sq=(tr, tc),
        promotion=promo,
        captured=target[0] if target else None,
        captured_color=target[1] if target else None,
    )
