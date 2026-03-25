# Taikyoku Shogi Engine

A complete engine for **Taikyoku Shogi** (大局将棋, "ultimate chess"), the largest known variant of shogi.

- **36 x 36** board (1,296 squares)
- **804 pieces** (402 per side)
- **209 piece types** with distinct movement patterns
- **92 additional promoted forms**

## Rules Overview

### Objective

Capture all of your opponent's **royal pieces** (King and Crown Prince). The game ends immediately when a player's last royal piece is captured. Unlike standard shogi, there is no check or checkmate — you may move your King into danger.

### Turns

Black (先手) moves first. Players alternate moving exactly one piece per turn. Passing is not allowed. Captured pieces are permanently removed (no drops).

### Movement

Pieces move orthogonally or diagonally. Most pieces have unique movement patterns combining:

| Type | Description |
|------|-------------|
| **Step** | Move exactly 1 square |
| **Slide** | Move any number of empty squares in a line |
| **Limited** | Move up to N squares in a line (2-7) |
| **Jump** | Leap to a square, bypassing intervening pieces |
| **Hook** | Slide along a line, optionally turn 90 degrees once, then continue sliding |
| **Area** | Take multiple steps per turn (Lion-type moves) |
| **Range capture** | Fly over pieces along a line, capturing all of lower rank |

See [PIECES.md](PIECES.md) for the complete movement reference for every piece.

### Promotion

The **promotion zone** is the 11 rows on the opponent's side of the board.

- A piece **may promote** when it moves into the promotion zone from outside, or when it captures an opponent's piece while inside the zone.
- A piece that entered the zone without promoting may only promote later by capturing inside the zone, or by leaving and re-entering.
- Promotion is **optional**, except for forward-only pieces (Pawn, Stone General, Iron General, Dog, Wood General, Incense Chariot, Ox Chariot, Fierce Tiger) which **must promote** upon reaching the farthest rank.
- Promotion is permanent — a piece cannot revert to its unpromoted form.

### Range Capture Ranking

Range-capturing pieces (Great General, Vice General, etc.) can fly over and capture all pieces along a line, but only pieces of **lower rank**:

1. King, Crown Prince — cannot be range-captured
2. Great General
3. Vice General
4. Flying General, Angle General, Fierce Dragon, Flying Crocodile

### Initial Setup

Each side's 402 pieces occupy 12 ranks. Black occupies the bottom of the board (rows 25-36); White mirrors from the top (rows 1-12). The King sits at the center of the back rank, flanked by the Crown Prince.

## Getting Started

### Requirements

- Python 3.9+
- Rust toolchain (for the fast Rust backend)
- `maturin` (`pip install maturin`)

### Build and Run

```bash
# Clone the repository
git clone https://github.com/jh85/taikyokushogi.git
cd taikyokushogi

# Create a virtual environment and install dependencies
python3 -m venv .venv
source .venv/bin/activate
pip install maturin

# Build the Rust backend
maturin develop --release

# Run the web GUI
python3 web_gui.py 8080
# Then open http://localhost:8080 in your browser
```

### Web GUI Features

- **Human vs Random** — play as Black or White against a random-move player
- **Random vs Random** — watch two random players with adjustable speed
- Click a piece to see legal moves highlighted on the board
- Score graph tracks material balance in real time
- Piece info panel shows movement details on hover

### Other Modes

```bash
# USI protocol mode (for GUI software)
source .venv/bin/activate
python3 -m taikyoku_engine

# Quick demo (board info + move count)
python3 -m taikyoku_engine demo

# Random game in terminal
python3 -m taikyoku_engine random 200
```

### Pure Python (no Rust)

If you cannot install Rust, the engine falls back to pure Python automatically. Everything works, just slower (~80x slower for search).

```bash
python3 web_gui.py 8080
```

## Project Structure

```
taikyokushogi/
  src/                    # Rust engine (PyO3)
    lib.rs                #   Public API + Python bindings
    python.rs             #   PyO3 bindings (behind feature flag)
    types.rs              #   Core types, ray tables
    pieces.rs             #   301 piece types, Betza parser
    board.rs              #   Board representation
    movegen.rs            #   Legal move generation
    eval.rs               #   Static evaluation
    search.rs             #   Alpha-beta search
  taikyoku_engine/        # Python engine (fallback)
    pieces.py             #   Piece data
    board.py              #   Board class
    movegen.py            #   Move generation
    evaluation.py         #   Evaluation
    search.py             #   Search
    usi.py                #   USI protocol
  web_gui.py              # Browser-based game GUI
  PIECES.md               # Complete piece movement reference
  Cargo.toml              # Rust project config
  pyproject.toml          # Python/maturin build config
```

## Performance

| Benchmark | Python | Rust | Speedup |
|---|---|---|---|
| Legal move generation | 1.35 ms | 0.28 ms | 5x |
| Depth-1 search | 145 ms | 1.9 ms | 76x |
| Depth-2 search | 3,017 ms | 38 ms | 79x |
| Depth-3 search | — | 4,954 ms | — |
| Random game (500 moves) | ~14 s | 29 ms | 500x |

## Using as a Rust Crate

Add to your `Cargo.toml`:

```toml
[dependencies]
taikyokushogi = "0.1"
```

```rust
use taikyokushogi::{Board, Color};

let mut board = Board::initial();
let moves = board.legal_moves();
println!("{} legal moves", moves.len());

board.apply(&moves[0]);
println!("Score: {}", board.material_score());
board.undo();

// Search
let result = board.search(2, 5000);
if let Some(mv) = result.best_move {
    println!("Best: {}, score: {}", mv, result.score);
}

// Piece info
let info = taikyokushogi::piece_info("LN").unwrap();
println!("{}: {} (area={}, igui={})", info.name, info.value, info.area_steps, info.has_igui);
```

## References

- [Taikyoku shogi — Wikipedia](https://en.wikipedia.org/wiki/Taikyoku_shogi)
