"""Taikyoku Shogi Engine - the largest known shogi variant (36x36 board, 804 pieces).

Uses Rust backend (taikyokushogi) when available, falls back to pure Python.
"""
try:
    from taikyokushogi import PyBoard as _RustBoard
    HAS_RUST = True
except ImportError:
    HAS_RUST = False
