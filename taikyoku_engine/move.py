"""Move representation for Taikyoku Shogi."""

from dataclasses import dataclass


@dataclass
class Move:
    """Represents a single move in the game."""
    from_sq: tuple[int, int]   # (row, col)
    to_sq: tuple[int, int]     # (row, col)
    promotion: bool = False
    captured: str | None = None      # abbreviation of captured piece (or None)
    captured_color: int | None = None
    # For area movers (lion): intermediate capture
    mid_sq: tuple[int, int] | None = None
    mid_captured: str | None = None
    mid_captured_color: int | None = None
    # For range capture: list of (row, col, piece, color) captured along the way
    range_captures: list | None = None
    # Igui (capture without moving)
    is_igui: bool = False

    def __str__(self):
        fr, fc = self.from_sq
        tr, tc = self.to_sq
        s = f"{_sq_name(fr, fc)}{_sq_name(tr, tc)}"
        if self.promotion:
            s += '+'
        return s

    def __repr__(self):
        return f"Move({self})"

    def __eq__(self, other):
        if not isinstance(other, Move):
            return False
        return (self.from_sq == other.from_sq and
                self.to_sq == other.to_sq and
                self.promotion == other.promotion and
                self.mid_sq == other.mid_sq and
                self.is_igui == other.is_igui)

    def __hash__(self):
        return hash((self.from_sq, self.to_sq, self.promotion,
                      self.mid_sq, self.is_igui))


def _sq_name(row, col):
    """Convert (row, col) to algebraic notation like '18a' (file + rank)."""
    # File: numbered 36 down to 1 (right to left from Black's view)
    file_num = 36 - col
    # Rank: lettered a-z, aa-aj (bottom to top)
    rank_num = 36 - row
    if rank_num <= 26:
        rank_str = chr(ord('a') + rank_num - 1)
    else:
        rank_str = 'a' + chr(ord('a') + rank_num - 27)
    return f"{file_num}{rank_str}"


def parse_sq_name(name):
    """Parse square name back to (row, col)."""
    # Find where digits end and letters begin
    i = 0
    while i < len(name) and name[i].isdigit():
        i += 1
    file_num = int(name[:i])
    rank_str = name[i:]

    col = 36 - file_num
    if len(rank_str) == 1:
        rank_num = ord(rank_str) - ord('a') + 1
    else:
        rank_num = 27 + ord(rank_str[1]) - ord('a')
    row = 36 - rank_num
    return row, col
