"""Main entry point for the Taikyoku Shogi engine."""

import sys
from .usi import USIHandler
from .board import TaikyokuBoard
from .movegen import generate_legal_moves, choose_random_move
from .tsfen import to_tsfen


def main():
    """Entry point: start USI mode or run a demo."""
    if len(sys.argv) > 1 and sys.argv[1] == 'demo':
        run_demo()
    elif len(sys.argv) > 1 and sys.argv[1] == 'random':
        run_random_game(max_moves=int(sys.argv[2]) if len(sys.argv) > 2 else 100)
    else:
        # Default: USI mode
        handler = USIHandler()
        handler.run()


def run_demo():
    """Demo: set up the board and show info."""
    board = TaikyokuBoard()
    board.setup_initial()

    print("=== Taikyoku Shogi Engine ===")
    print(f"Board size: {board.BOARD_SIZE if hasattr(board, 'BOARD_SIZE') else 36}x36")

    b_count = len(board.piece_positions[0])
    w_count = len(board.piece_positions[1])
    print(f"Black pieces: {b_count}")
    print(f"White pieces: {w_count}")
    print(f"Total: {b_count + w_count}")
    print()

    print("TSFEN of initial position:")
    print(to_tsfen(board))
    print()

    print("Generating legal moves for Black...")
    moves = generate_legal_moves(board)
    print(f"Total legal moves: {len(moves)}")
    print()

    print("First 20 moves:")
    for m in moves[:20]:
        print(f"  {m}")
    if len(moves) > 20:
        print(f"  ... and {len(moves) - 20} more")
    print()

    print("Board (top rows - White's side):")
    lines = board.display().split('\n')
    for line in lines[:5]:
        print(line)
    print("  ...")
    for line in lines[-5:]:
        print(line)


def run_random_game(max_moves=100):
    """Play a random game."""
    board = TaikyokuBoard()
    board.setup_initial()

    print(f"Starting random game (max {max_moves} moves)...")
    print(f"Initial pieces: Black={len(board.piece_positions[0])}, "
          f"White={len(board.piece_positions[1])}")
    print()

    for i in range(max_moves):
        result = board.get_game_result()
        if result:
            print(f"\nGame over after {i} moves: {result}")
            return

        move = choose_random_move(board)
        if move is None:
            side = "Black" if board.side_to_move == 0 else "White"
            print(f"\n{side} has no legal moves! (stalemate)")
            return

        side = "Black" if board.side_to_move == 0 else "White"
        capture_info = f" captures {move.captured}" if move.captured else ""
        promo_info = " (promotes)" if move.promotion else ""
        print(f"Move {i+1}: {side} {move}{capture_info}{promo_info}")

        board.apply_move(move)

    b_count = len(board.piece_positions[0])
    w_count = len(board.piece_positions[1])
    print(f"\nGame stopped after {max_moves} moves.")
    print(f"Remaining pieces: Black={b_count}, White={w_count}")


if __name__ == '__main__':
    main()
