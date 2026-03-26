#!/usr/bin/env python3
"""Taikyoku Shogi Web GUI - play in browser against random player or watch random vs random."""

import json
import sys
import os
import threading
import time as _time
from http.server import HTTPServer, BaseHTTPRequestHandler
from urllib.parse import urlparse, parse_qs

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

# Try Rust backend first, fall back to pure Python
try:
    import taikyokushogi
    USE_RUST = True
    print("Using Rust backend (taikyokushogi)")
except ImportError:
    USE_RUST = False
    print("Rust backend not found, using pure Python")

if not USE_RUST:
    from taikyoku_engine.board import TaikyokuBoard
    from taikyoku_engine.pieces import (
        BOARD_SIZE, BLACK, WHITE, PIECE_NAME, PIECE_VALUE, PROMOTES_TO, MOVEMENTS,
    )
    from taikyoku_engine.movegen import generate_legal_moves, choose_random_move
    from taikyoku_engine.move import Move
else:
    BOARD_SIZE = taikyokushogi.BOARD_SIZE
    BLACK = taikyokushogi.BLACK
    WHITE = taikyokushogi.WHITE

# ============================================================
# Game State (global, protected by lock)
# ============================================================
game_lock = threading.Lock()


class GameState:
    def __init__(self):
        if USE_RUST:
            self.board = taikyokushogi.PyBoard()
        else:
            self.board = TaikyokuBoard()
        self.board.setup_initial()
        self.mode = 'human_vs_random'
        self.human_color = BLACK
        self.move_log = []
        self.score_history = [self._score()]
        self.half_move = 0
        self.selected = None
        self.game_over = False
        # Game record
        self.record = []          # list of {move_num, side, piece, from, to, promo, time_s, score}
        self.last_move_time = _time.monotonic()
        self.game_start_time = _time.strftime('%Y-%m-%d %H:%M:%S')

    def reset(self, mode='human_vs_random', human_color=BLACK):
        if USE_RUST:
            self.board = taikyokushogi.PyBoard()
        else:
            self.board = TaikyokuBoard()
        self.board.setup_initial()
        self.mode = mode
        self.human_color = human_color
        self.move_log = []
        self.score_history = [self._score()]
        self.half_move = 0
        self.selected = None
        self.game_over = False
        self.record = []
        self.last_move_time = _time.monotonic()
        self.game_start_time = _time.strftime('%Y-%m-%d %H:%M:%S')

    def record_move(self, side, piece_abbrev, fr, fc, tr, tc, promotion, score):
        now = _time.monotonic()
        elapsed = round(now - self.last_move_time, 2)
        self.last_move_time = now
        self.record.append({
            'n': self.half_move,
            'side': side,
            'piece': piece_abbrev,
            'from': (fr, fc),
            'to': (tr, tc),
            'promo': promotion,
            'time': elapsed,
            'score': score,
        })

    def _score(self):
        if USE_RUST:
            return self.board.score()
        score = 0
        for (r, c), piece in self.board.piece_positions[BLACK].items():
            score += PIECE_VALUE.get(piece, 1000)
        for (r, c), piece in self.board.piece_positions[WHITE].items():
            score -= PIECE_VALUE.get(piece, 1000)
        return score


game = GameState()


def board_to_json(board):
    """Serialize board state to JSON-compatible dict."""
    if USE_RUST:
        cells = []
        for r in range(BOARD_SIZE):
            row = []
            for c in range(BOARD_SIZE):
                cell = board.at(r, c)
                if cell is None:
                    row.append(None)
                else:
                    piece, color = cell
                    row.append({'piece': piece, 'color': color,
                                'name': taikyokushogi.piece_name_py(piece)})
            cells.append(row)
        result = board.game_result()
        return {
            'board': cells,
            'side_to_move': board.side_to_move,
            'move_number': board.move_number,
            'game_result': result,
            'black_pieces': board.black_piece_count(),
            'white_pieces': board.white_piece_count(),
        }
    else:
        cells = []
        for r in range(BOARD_SIZE):
            row = []
            for c in range(BOARD_SIZE):
                cell = board.at(r, c)
                if cell is None:
                    row.append(None)
                else:
                    piece, color = cell
                    row.append({'piece': piece, 'color': color,
                                'name': PIECE_NAME.get(piece, piece)})
            cells.append(row)
        result = board.get_game_result()
        return {
            'board': cells,
            'side_to_move': board.side_to_move,
            'move_number': board.move_number,
            'game_result': result,
            'black_pieces': len(board.piece_positions[BLACK]),
            'white_pieces': len(board.piece_positions[WHITE]),
        }


def moves_for_square(board, r, c):
    if USE_RUST:
        return board.moves_from_py(r, c)
    cell = board.at(r, c)
    if cell is None: return []
    piece, color = cell
    if color != board.side_to_move: return []
    all_moves = generate_legal_moves(board)
    result = []
    seen = set()
    for m in all_moves:
        if m.from_sq == (r, c):
            key = (m.to_sq, m.promotion, m.is_igui)
            if key not in seen:
                seen.add(key)
                cap_name = PIECE_NAME.get(m.captured, m.captured) if m.captured else None
                result.append({'to': list(m.to_sq), 'promotion': m.promotion,
                               'is_igui': m.is_igui, 'captured': cap_name})
    return result


def find_matching_move(board, fr, fc, tr, tc, promotion=False):
    if USE_RUST:
        return board.apply_move_py(fr, fc, tr, tc, promotion)
    all_moves = generate_legal_moves(board)
    for m in all_moves:
        if m.from_sq == (fr, fc) and m.to_sq == (tr, tc) and m.promotion == promotion:
            return m
    return None


# ============================================================
# HTTP Handler
# ============================================================
class GameHandler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass  # Suppress request logging

    def _send_json(self, data, status=200):
        body = json.dumps(data).encode()
        self.send_response(status)
        self.send_header('Content-Type', 'application/json')
        self.send_header('Content-Length', len(body))
        self.send_header('Access-Control-Allow-Origin', '*')
        self.end_headers()
        self.wfile.write(body)

    def _send_html(self, html):
        body = html.encode()
        self.send_response(200)
        self.send_header('Content-Type', 'text/html; charset=utf-8')
        self.send_header('Content-Length', len(body))
        self.end_headers()
        self.wfile.write(body)

    def _read_body(self):
        length = int(self.headers.get('Content-Length', 0))
        if length:
            return json.loads(self.rfile.read(length))
        return {}

    def do_GET(self):
        parsed = urlparse(self.path)
        path = parsed.path
        params = parse_qs(parsed.query)

        if path == '/' or path == '/index.html':
            self._send_html(HTML_PAGE)

        elif path == '/api/state':
            with game_lock:
                data = board_to_json(game.board)
                data['mode'] = game.mode
                data['human_color'] = game.human_color
                data['move_count'] = len(game.move_log)
                data['move_log'] = game.move_log[-50:]
                data['move_log_offset'] = max(0, len(game.move_log) - 50)
                data['score_history'] = game.score_history
                data['game_over'] = game.game_over
            self._send_json(data)

        elif path == '/api/moves':
            r = int(params.get('r', ['-1'])[0])
            c = int(params.get('c', ['-1'])[0])
            with game_lock:
                moves = moves_for_square(game.board, r, c)
            self._send_json({'moves': moves, 'from': [r, c]})

        elif path == '/api/piece-info':
            abbrev = params.get('abbrev', [''])[0]
            if USE_RUST:
                self._send_json(taikyokushogi.piece_info_py(abbrev))
            else:
                name = PIECE_NAME.get(abbrev, abbrev)
                value = PIECE_VALUE.get(abbrev, 0)
                promo = PROMOTES_TO.get(abbrev)
                promo_name = PIECE_NAME.get(promo, promo) if promo else None
                mov = MOVEMENTS.get(abbrev, {})
                slide_count = len(mov.get('slides', []))
                jump_count = len(mov.get('jumps', []))
                specials = []
                if mov.get('hook'): specials.append(f"hook ({mov['hook']})")
                if mov.get('area'): specials.append(f"area ({mov['area']})")
                if mov.get('range_capture'): specials.append("range capture")
                if mov.get('igui'): specials.append("igui")
                self._send_json({
                    'abbrev': abbrev, 'name': name, 'value': value,
                    'promotes_to': promo_name,
                    'slide_directions': slide_count,
                    'jump_destinations': jump_count,
                    'specials': specials,
                })

        elif path == '/api/record':
            with game_lock:
                text = _build_game_record()
            body = text.encode('utf-8')
            self.send_response(200)
            self.send_header('Content-Type', 'text/plain; charset=utf-8')
            self.send_header('Content-Disposition', 'attachment; filename="taikyoku_game.tsv"')
            self.send_header('Content-Length', len(body))
            self.end_headers()
            self.wfile.write(body)

        else:
            self.send_error(404)

    def do_POST(self):
        path = urlparse(self.path).path
        body = self._read_body()

        if path == '/api/new-game':
            mode = body.get('mode', 'human_vs_random')
            human_color = body.get('human_color', BLACK)
            with game_lock:
                game.reset(mode=mode, human_color=human_color)
            self._send_json({'ok': True})

        elif path == '/api/move':
            fr, fc = body['from']
            tr, tc = body['to']
            promotion = body.get('promotion', False)
            with game_lock:
                if game.game_over:
                    self._send_json({'ok': False, 'error': 'Game is over'})
                    return
                game.half_move += 1
                side = 'Black' if game.board.side_to_move == BLACK else 'White'
                piece = game.board.at(fr, fc)
                pname = piece[0] if piece else '?'
                promo_s = "+" if promotion else ""
                if USE_RUST:
                    ok = game.board.apply_move_py(fr, fc, tr, tc, promotion)
                    if not ok:
                        game.half_move -= 1
                        self._send_json({'ok': False, 'error': 'Illegal move'})
                        return
                else:
                    m = find_matching_move(game.board, fr, fc, tr, tc, promotion)
                    if m is None:
                        game.half_move -= 1
                        self._send_json({'ok': False, 'error': 'Illegal move'})
                        return
                    game.board.apply_move(m)
                score = game._score()
                game.score_history.append(score)
                game.record_move(side, pname, fr, fc, tr, tc, promotion, score)
                entry = f"{game.half_move}. {side}: {pname} {_sq(fr,fc)}-{_sq(tr,tc)}{promo_s}"
                game.move_log.append(entry)
                result = board_to_json(game.board)['game_result']
                if result:
                    game.game_over = True
                    game.move_log.append(f"** Game over: {result} **")
                data = board_to_json(game.board)
                data['move_log'] = game.move_log[-50:]
                data['move_log_offset'] = max(0, len(game.move_log) - 50)
                data['score_history'] = game.score_history
                data['game_over'] = game.game_over
            self._send_json({'ok': True, **data})

        elif path == '/api/ai-move':
            depth = body.get('depth', 0)
            time_limit = body.get('time_limit', 30000)  # ms, default 30s
            with game_lock:
                if game.game_over:
                    self._send_json({'ok': False, 'error': 'Game is over'})
                    return
                game.half_move += 1
                side = 'Black' if game.board.side_to_move == BLACK else 'White'
                search_ms = 0
                search_nodes = 0
                if depth == 0:
                    # Random move
                    if USE_RUST:
                        rm = game.board.random_move_py()
                    else:
                        m = choose_random_move(game.board)
                        rm = (m.from_sq[0], m.from_sq[1], m.to_sq[0], m.to_sq[1], m.promotion) if m else None
                    if rm is None:
                        game.half_move -= 1
                        game.game_over = True
                        game.move_log.append("No legal moves - stalemate")
                        self._send_json({'ok': False, 'error': 'No legal moves'})
                        return
                    fr, fc, tr, tc, promotion = rm
                else:
                    # AI search
                    if USE_RUST:
                        result = game.board.search_py(depth, time_limit)
                        mv, search_score, search_nodes, search_ms = result
                    else:
                        from taikyoku_engine.search import search as py_search
                        result = py_search(game.board, depth=depth, time_limit_ms=time_limit)
                        mv = (result.best_move.from_sq[0], result.best_move.from_sq[1],
                              result.best_move.to_sq[0], result.best_move.to_sq[1],
                              result.best_move.promotion) if result.best_move else None
                        search_nodes = result.nodes
                        search_ms = result.time_ms
                    if mv is None:
                        game.half_move -= 1
                        game.game_over = True
                        game.move_log.append("No legal moves - stalemate")
                        self._send_json({'ok': False, 'error': 'No legal moves'})
                        return
                    fr, fc, tr, tc, promotion = mv
                piece = game.board.at(fr, fc)
                pname = piece[0] if piece else '?'
                if USE_RUST:
                    game.board.apply_move_py(fr, fc, tr, tc, promotion)
                else:
                    mm = find_matching_move(game.board, fr, fc, tr, tc, promotion)
                    if mm: game.board.apply_move(mm)
                promo_s = "+" if promotion else ""
                score = game._score()
                game.score_history.append(score)
                game.record_move(side, pname, fr, fc, tr, tc, promotion, score)
                depth_s = f" d{depth}" if depth > 0 else ""
                time_s = f" {search_ms}ms" if search_ms else ""
                entry = f"{game.half_move}. {side}: {pname} {_sq(fr,fc)}-{_sq(tr,tc)}{promo_s}{depth_s}{time_s}"
                game.move_log.append(entry)
                result = board_to_json(game.board)['game_result']
                if result:
                    game.game_over = True
                    game.move_log.append(f"** Game over: {result} **")
                data = board_to_json(game.board)
                data['move_log'] = game.move_log[-50:]
                data['move_log_offset'] = max(0, len(game.move_log) - 50)
                data['score_history'] = game.score_history
                data['game_over'] = game.game_over
                data['last_move'] = {'from': [fr, fc], 'to': [tr, tc]}
            self._send_json({'ok': True, **data})

        elif path == '/api/undo':
            with game_lock:
                if USE_RUST:
                    ok = game.board.undo()
                else:
                    ok = bool(game.board.move_history)
                    if ok:
                        game.board.undo_move()
                if ok:
                    if game.move_log:
                        game.move_log.pop()
                    if len(game.score_history) > 1:
                        game.score_history.pop()
                    if game.half_move > 0:
                        game.half_move -= 1
                    game.game_over = False
                    self._send_json({'ok': True})
                else:
                    self._send_json({'ok': False, 'error': 'Nothing to undo'})

        else:
            self.send_error(404)

    def do_OPTIONS(self):
        self.send_response(204)
        self.send_header('Access-Control-Allow-Origin', '*')
        self.send_header('Access-Control-Allow-Methods', 'GET, POST, OPTIONS')
        self.send_header('Access-Control-Allow-Headers', 'Content-Type')
        self.end_headers()


def _sq(r, c):
    """Format square coordinate."""
    return f"({r},{c})"


def _build_game_record():
    """Build a game record as a tab-separated text file."""
    lines = []
    lines.append(f"# Taikyoku Shogi Game Record")
    lines.append(f"# Date: {game.game_start_time}")
    lines.append(f"# Mode: {game.mode}")
    result = board_to_json(game.board).get('game_result')
    lines.append(f"# Result: {result or 'in progress'}")
    lines.append(f"# Moves: {game.half_move}")
    lines.append(f"# Final score (Black perspective): {game._score()}")
    lines.append(f"#")
    lines.append(f"# move\tside\tpiece\tfrom_r\tfrom_c\tto_r\tto_c\tpromo\ttime_s\tscore")

    for rec in game.record:
        fr, fc = rec['from']
        tr, tc = rec['to']
        promo = '+' if rec['promo'] else ''
        lines.append(f"{rec['n']}\t{rec['side']}\t{rec['piece']}\t{fr}\t{fc}\t{tr}\t{tc}\t{promo}\t{rec['time']}\t{rec['score']}")

    lines.append('')
    return '\n'.join(lines)


# ============================================================
# HTML Frontend (embedded)
# ============================================================
HTML_PAGE = r"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>Taikyoku Shogi</title>
<style>
* { margin: 0; padding: 0; box-sizing: border-box; }
body {
    font-family: 'Segoe UI', system-ui, sans-serif;
    background: #1a1a2e;
    color: #e0e0e0;
    min-height: 100vh;
}
.header {
    background: #16213e;
    padding: 8px 20px;
    display: flex;
    align-items: center;
    gap: 16px;
    border-bottom: 2px solid #0f3460;
    flex-wrap: wrap;
}
.header h1 {
    font-size: 18px;
    color: #e94560;
    white-space: nowrap;
}
.controls {
    display: flex;
    gap: 8px;
    align-items: center;
    flex-wrap: wrap;
}
.controls select, .controls button {
    padding: 4px 10px;
    border-radius: 4px;
    border: 1px solid #0f3460;
    background: #1a1a2e;
    color: #e0e0e0;
    font-size: 12px;
    cursor: pointer;
}
.controls button { background: #0f3460; }
.controls button:hover { background: #e94560; }
.controls button.active { background: #e94560; }
.controls label { font-size: 12px; color: #aaa; }
.status-bar {
    font-size: 12px;
    color: #aaa;
    margin-left: auto;
    text-align: right;
    min-width: 200px;
}
.main {
    display: flex;
    gap: 12px;
    padding: 10px;
    justify-content: center;
    align-items: flex-start;
}
/* Board */
.board-wrap {
    overflow: auto;
    max-height: calc(100vh - 60px);
    border: 2px solid #0f3460;
    background: #0a0a1a;
    flex-shrink: 0;
}
.board {
    display: grid;
    grid-template-columns: 20px repeat(36, var(--cell));
    grid-template-rows: repeat(36, var(--cell)) 20px;
    gap: 0;
    --cell: 28px;
    font-size: 8px;
}
.coord {
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 7px;
    color: #666;
    user-select: none;
}
.cell {
    width: var(--cell);
    height: var(--cell);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    user-select: none;
    font-weight: 600;
    position: relative;
    transition: background 0.1s;
    line-height: 1;
    letter-spacing: -0.5px;
}
.cell.light { background: #c8b07a; }
.cell.dark { background: #a68a5b; }
.cell.black-piece { color: #1a0505; }
.cell.white-piece { color: #f5f5ff; }
.cell.white-piece::after {
    content: '';
    position: absolute;
    bottom: 1px;
    left: 50%;
    transform: translateX(-50%);
    width: 60%;
    height: 2px;
    background: #ccccff;
    border-radius: 1px;
}
.cell.selected { background: #ffd700 !important; }
.cell.legal-target { background: #5cb85c !important; cursor: crosshair; }
.cell.legal-target.has-enemy { background: #d9534f !important; }
.cell.last-from { box-shadow: inset 0 0 0 2px #4a90d9; }
.cell.last-to { box-shadow: inset 0 0 0 2px #4a90d9; }
.cell:hover { filter: brightness(1.2); }
.cell .promo-dot {
    position: absolute;
    top: 1px;
    right: 1px;
    width: 3px;
    height: 3px;
    background: #e94560;
    border-radius: 50%;
}
/* Sidebar */
.sidebar {
    width: 260px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex-shrink: 0;
    max-height: calc(100vh - 60px);
}
.panel {
    background: #16213e;
    border: 1px solid #0f3460;
    border-radius: 6px;
    padding: 10px;
}
.panel h3 {
    font-size: 12px;
    color: #e94560;
    margin-bottom: 6px;
    border-bottom: 1px solid #0f3460;
    padding-bottom: 4px;
}
.panel .info-row {
    display: flex;
    justify-content: space-between;
    font-size: 11px;
    padding: 2px 0;
}
.panel .info-row .label { color: #888; }
.move-log {
    flex: 1;
    overflow-y: auto;
    max-height: 400px;
}
.move-log .entry {
    font-size: 10px;
    padding: 1px 4px;
    font-family: monospace;
    border-bottom: 1px solid #0a0a1a;
}
.move-log .entry:nth-child(odd) { background: rgba(255,255,255,0.02); }
.move-log .entry .move-num { color: #666; margin-right: 2px; }
/* Score graph */
.score-graph-wrap {
    position: relative;
    background: #0d1b2a;
    border-radius: 4px;
    overflow: hidden;
    margin-top: 6px;
}
.score-graph-wrap canvas {
    display: block;
    width: 100%;
}
.score-label {
    font-size: 9px;
    color: #666;
    display: flex;
    justify-content: space-between;
    padding: 0 2px;
}
.score-current {
    font-size: 12px;
    font-weight: 700;
    text-align: center;
    padding: 4px 0 2px;
}
.score-current.positive { color: #5cb85c; }
.score-current.negative { color: #d9534f; }
.score-current.zero { color: #888; }
.piece-info {
    font-size: 11px;
    min-height: 80px;
}
.piece-info .piece-name { font-size: 14px; font-weight: 700; color: #fff; }
.piece-info .piece-detail { color: #aaa; margin-top: 4px; }
/* Promotion dialog */
.promo-dialog {
    display: none;
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    background: #16213e;
    border: 2px solid #e94560;
    border-radius: 8px;
    padding: 20px;
    z-index: 1000;
    text-align: center;
}
.promo-dialog.show { display: block; }
.promo-dialog h3 { margin-bottom: 12px; color: #e94560; }
.promo-dialog button {
    padding: 8px 24px;
    margin: 4px;
    border-radius: 4px;
    border: 1px solid #0f3460;
    background: #0f3460;
    color: #e0e0e0;
    cursor: pointer;
    font-size: 14px;
}
.promo-dialog button:hover { background: #e94560; }
.overlay {
    display: none;
    position: fixed;
    top: 0; left: 0; right: 0; bottom: 0;
    background: rgba(0,0,0,0.5);
    z-index: 999;
}
.overlay.show { display: block; }
/* Speed selector */
.speed-control { display: flex; align-items: center; gap: 4px; }
.speed-control input[type=range] { width: 80px; }
</style>
</head>
<body>

<div class="header">
    <h1>Taikyoku Shogi</h1>
    <div class="controls">
        <label>Mode:</label>
        <select id="mode-select">
            <option value="human_vs_ai">Human vs AI</option>
            <option value="human_vs_random">Human vs Random</option>
            <option value="random_vs_random">Random vs Random</option>
            <option value="ai_vs_ai">AI vs AI</option>
        </select>
        <label>Play as:</label>
        <select id="color-select">
            <option value="0">Black (first)</option>
            <option value="1">White (second)</option>
        </select>
        <label>AI:</label>
        <select id="ai-depth">
            <option value="0">Random</option>
            <option value="1">Depth 1</option>
            <option value="2" selected>Depth 2</option>
            <option value="3">Depth 3</option>
            <option value="4">Depth 4 (slow)</option>
            <option value="5">Depth 5 (very slow)</option>
        </select>
        <button onclick="newGame()">New Game</button>
        <button onclick="undoMove()">Undo</button>
        <button id="auto-btn" onclick="toggleAuto()">Auto Play</button>
        <button onclick="downloadRecord()">Save Record</button>
        <div class="speed-control">
            <label>Speed:</label>
            <input type="range" id="speed" min="100" max="3000" value="800" step="100">
            <span id="speed-label">800ms</span>
        </div>
    </div>
    <div class="status-bar" id="status">Loading...</div>
</div>

<div class="main">
    <div class="board-wrap">
        <div class="board" id="board"></div>
    </div>
    <div class="sidebar">
        <div class="panel">
            <h3>Game Info</h3>
            <div class="info-row"><span class="label">Turn:</span><span id="info-turn">Black</span></div>
            <div class="info-row"><span class="label">Move #:</span><span id="info-move">1</span></div>
            <div class="info-row"><span class="label">Black pieces:</span><span id="info-black">402</span></div>
            <div class="info-row"><span class="label">White pieces:</span><span id="info-white">402</span></div>
            <div class="info-row"><span class="label">Result:</span><span id="info-result">-</span></div>
        </div>
        <div class="panel piece-info" id="piece-info-panel">
            <h3>Piece Info</h3>
            <div id="piece-info-content">Click a piece to see details</div>
        </div>
        <div class="panel" style="flex:1; display:flex; flex-direction:column; min-height:0;">
            <h3>Move Log</h3>
            <div class="move-log" id="move-log"></div>
        </div>
        <div class="panel">
            <h3>Score (Black's Perspective)</h3>
            <div class="score-current" id="score-current">0</div>
            <div class="score-graph-wrap">
                <canvas id="score-canvas" width="238" height="100"></canvas>
            </div>
            <div class="score-label">
                <span>Move 0</span>
                <span id="score-x-end">-</span>
            </div>
        </div>
    </div>
</div>

<div class="overlay" id="overlay"></div>
<div class="promo-dialog" id="promo-dialog">
    <h3>Promote this piece?</h3>
    <p id="promo-text"></p>
    <button onclick="doPromo(true)">Yes, Promote</button>
    <button onclick="doPromo(false)">No, Keep</button>
</div>

<script>
const SIZE = 36;
let boardState = null;
let selectedSq = null;
let legalMoves = [];
let lastMove = null;
let autoPlay = false;
let autoTimer = null;
let pendingPromo = null; // {from, to}
let gameMode = 'human_vs_random';
let humanColor = 0;

// Build the board grid
function buildBoard() {
    const el = document.getElementById('board');
    el.innerHTML = '';
    // Column headers
    el.appendChild(makeCoord(''));
    for (let c = 0; c < SIZE; c++) {
        const d = makeCoord(SIZE - c);
        el.appendChild(d);
    }
    // Rows
    for (let r = 0; r < SIZE; r++) {
        // Row label
        el.appendChild(makeCoord(SIZE - r));
        for (let c = 0; c < SIZE; c++) {
            const cell = document.createElement('div');
            cell.className = 'cell ' + ((r + c) % 2 === 0 ? 'light' : 'dark');
            cell.dataset.r = r;
            cell.dataset.c = c;
            cell.id = `cell-${r}-${c}`;
            cell.addEventListener('click', () => onCellClick(r, c));
            cell.addEventListener('mouseenter', () => onCellHover(r, c));
            el.appendChild(cell);
        }
    }
    // Bottom row label
    el.appendChild(makeCoord(''));
    for (let c = 0; c < SIZE; c++) {
        el.appendChild(makeCoord(SIZE - c));
    }
}

function makeCoord(text) {
    const d = document.createElement('div');
    d.className = 'coord';
    d.textContent = text;
    return d;
}

function renderBoard(state) {
    boardState = state;
    for (let r = 0; r < SIZE; r++) {
        for (let c = 0; c < SIZE; c++) {
            const cell = document.getElementById(`cell-${r}-${c}`);
            if (!cell) continue;
            // Reset classes
            cell.className = 'cell ' + ((r + c) % 2 === 0 ? 'light' : 'dark');
            cell.innerHTML = '';
            const piece = state.board[r][c];
            if (piece) {
                cell.textContent = piece.piece;
                if (piece.color === 0) {
                    cell.classList.add('black-piece');
                } else {
                    cell.classList.add('white-piece');
                }
            }
            // Highlight selected
            if (selectedSq && selectedSq[0] === r && selectedSq[1] === c) {
                cell.classList.add('selected');
            }
            // Highlight legal targets
            const lm = legalMoves.find(m => m.to[0] === r && m.to[1] === c);
            if (lm) {
                cell.classList.add('legal-target');
                if (lm.captured) cell.classList.add('has-enemy');
            }
            // Highlight last move
            if (lastMove) {
                if (lastMove.from[0] === r && lastMove.from[1] === c) cell.classList.add('last-from');
                if (lastMove.to[0] === r && lastMove.to[1] === c) cell.classList.add('last-to');
            }
        }
    }
    // Update info panel
    document.getElementById('info-turn').textContent = state.side_to_move === 0 ? 'Black' : 'White';
    document.getElementById('info-move').textContent = state.move_number;
    document.getElementById('info-black').textContent = state.black_pieces;
    document.getElementById('info-white').textContent = state.white_pieces;
    document.getElementById('info-result').textContent = state.game_result || '-';
    // Move log
    const logEl = document.getElementById('move-log');
    if (state.move_log) {
        logEl.innerHTML = state.move_log.map(e => `<div class="entry">${e}</div>`).join('');
        logEl.scrollTop = logEl.scrollHeight;
    }
    // Status
    let statusText = `${state.black_pieces} vs ${state.white_pieces} pieces`;
    if (state.game_result) {
        statusText = state.game_result.replace('_', ' ');
        stopAuto();
    }
    document.getElementById('status').textContent = statusText;
    // Score graph
    if (state.score_history) {
        drawScoreGraph(state.score_history);
    }
}

function drawScoreGraph(scores) {
    const canvas = document.getElementById('score-canvas');
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    const W = canvas.width;
    const H = canvas.height;
    ctx.clearRect(0, 0, W, H);

    if (scores.length < 1) return;

    const current = scores[scores.length - 1];
    const curEl = document.getElementById('score-current');
    const sign = current > 0 ? '+' : '';
    curEl.textContent = `${sign}${current}`;
    curEl.className = 'score-current ' + (current > 0 ? 'positive' : current < 0 ? 'negative' : 'zero');

    document.getElementById('score-x-end').textContent = `Move ${scores.length - 1}`;

    // Compute Y range
    let minS = Math.min(...scores);
    let maxS = Math.max(...scores);
    if (minS === maxS) { minS -= 1000; maxS += 1000; }
    const pad = (maxS - minS) * 0.1;
    minS -= pad;
    maxS += pad;

    const n = scores.length;
    const xStep = n > 1 ? W / (n - 1) : W;

    // Draw zero line
    const zeroY = H - ((0 - minS) / (maxS - minS)) * H;
    ctx.strokeStyle = '#334';
    ctx.lineWidth = 1;
    ctx.setLineDash([4, 4]);
    ctx.beginPath();
    ctx.moveTo(0, zeroY);
    ctx.lineTo(W, zeroY);
    ctx.stroke();
    ctx.setLineDash([]);

    // Fill areas
    if (n > 1) {
        // Positive area (green)
        ctx.beginPath();
        ctx.moveTo(0, zeroY);
        for (let i = 0; i < n; i++) {
            const x = i * xStep;
            const y = H - ((scores[i] - minS) / (maxS - minS)) * H;
            const clampY = Math.min(y, zeroY);
            ctx.lineTo(x, clampY);
        }
        ctx.lineTo((n - 1) * xStep, zeroY);
        ctx.closePath();
        ctx.fillStyle = 'rgba(92, 184, 92, 0.2)';
        ctx.fill();

        // Negative area (red)
        ctx.beginPath();
        ctx.moveTo(0, zeroY);
        for (let i = 0; i < n; i++) {
            const x = i * xStep;
            const y = H - ((scores[i] - minS) / (maxS - minS)) * H;
            const clampY = Math.max(y, zeroY);
            ctx.lineTo(x, clampY);
        }
        ctx.lineTo((n - 1) * xStep, zeroY);
        ctx.closePath();
        ctx.fillStyle = 'rgba(217, 83, 79, 0.2)';
        ctx.fill();
    }

    // Draw score line
    ctx.strokeStyle = '#e0e0e0';
    ctx.lineWidth = 1.5;
    ctx.beginPath();
    for (let i = 0; i < n; i++) {
        const x = i * xStep;
        const y = H - ((scores[i] - minS) / (maxS - minS)) * H;
        if (i === 0) ctx.moveTo(x, y);
        else ctx.lineTo(x, y);
    }
    ctx.stroke();

    // Draw current score dot
    if (n > 0) {
        const lastX = (n - 1) * xStep;
        const lastY = H - ((scores[n - 1] - minS) / (maxS - minS)) * H;
        ctx.fillStyle = current >= 0 ? '#5cb85c' : '#d9534f';
        ctx.beginPath();
        ctx.arc(lastX, lastY, 3, 0, Math.PI * 2);
        ctx.fill();
    }
}

async function fetchState() {
    const res = await fetch('/api/state');
    const data = await res.json();
    gameMode = data.mode;
    humanColor = data.human_color;
    renderBoard(data);
}

async function onCellClick(r, c) {
    if (!boardState) return;
    if (boardState.game_result) return;

    // In non-human modes, clicks do nothing
    if (gameMode === 'random_vs_random' || gameMode === 'ai_vs_ai') return;

    // Only allow clicks when it's human's turn
    if (boardState.side_to_move !== humanColor) return;

    // Check if clicking a legal target
    const target = legalMoves.find(m => m.to[0] === r && m.to[1] === c);
    if (target && selectedSq) {
        // Check if both promote and non-promote options exist
        const promoOption = legalMoves.find(m => m.to[0] === r && m.to[1] === c && m.promotion);
        const noPromoOption = legalMoves.find(m => m.to[0] === r && m.to[1] === c && !m.promotion);
        if (promoOption && noPromoOption) {
            // Ask user
            pendingPromo = {from: selectedSq, to: [r, c]};
            const piece = boardState.board[selectedSq[0]][selectedSq[1]];
            document.getElementById('promo-text').textContent =
                `${piece.name} (${piece.piece}) captures and can promote.`;
            document.getElementById('promo-dialog').classList.add('show');
            document.getElementById('overlay').classList.add('show');
            return;
        }
        await makeMove(selectedSq[0], selectedSq[1], r, c, target.promotion);
        return;
    }

    // Select a piece
    const piece = boardState.board[r][c];
    if (piece && piece.color === humanColor) {
        selectedSq = [r, c];
        // Fetch legal moves for this piece
        const res = await fetch(`/api/moves?r=${r}&c=${c}`);
        const data = await res.json();
        legalMoves = data.moves || [];
        renderBoard(boardState);
        // Show piece info
        showPieceInfo(piece.piece);
    } else {
        selectedSq = null;
        legalMoves = [];
        renderBoard(boardState);
    }
}

async function onCellHover(r, c) {
    if (!boardState) return;
    const piece = boardState.board[r][c];
    if (piece) {
        showPieceInfo(piece.piece);
    }
}

async function showPieceInfo(abbrev) {
    const res = await fetch(`/api/piece-info?abbrev=${encodeURIComponent(abbrev)}`);
    const info = await res.json();
    let html = `<div class="piece-name">${info.name}</div>`;
    html += `<div class="piece-detail">Abbrev: ${info.abbrev}</div>`;
    html += `<div class="piece-detail">Value: ${info.value}</div>`;
    if (info.promotes_to) html += `<div class="piece-detail">Promotes to: ${info.promotes_to}</div>`;
    html += `<div class="piece-detail">Slides: ${info.slide_directions} dirs, Jumps: ${info.jump_destinations}</div>`;
    if (info.specials.length > 0) html += `<div class="piece-detail">Special: ${info.specials.join(', ')}</div>`;
    document.getElementById('piece-info-content').innerHTML = html;
}

async function doPromo(yes) {
    document.getElementById('promo-dialog').classList.remove('show');
    document.getElementById('overlay').classList.remove('show');
    if (pendingPromo) {
        await makeMove(pendingPromo.from[0], pendingPromo.from[1],
                       pendingPromo.to[0], pendingPromo.to[1], yes);
        pendingPromo = null;
    }
}

function getAiDepth() {
    return parseInt(document.getElementById('ai-depth').value);
}

function isHumanMode() {
    const mode = document.getElementById('mode-select').value;
    return mode === 'human_vs_random' || mode === 'human_vs_ai';
}

async function makeMove(fr, fc, tr, tc, promotion) {
    const res = await fetch('/api/move', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({from: [fr, fc], to: [tr, tc], promotion})
    });
    const data = await res.json();
    if (data.ok) {
        lastMove = {from: [fr, fc], to: [tr, tc]};
        selectedSq = null;
        legalMoves = [];
        renderBoard(data);
        // Trigger AI/random response after short delay
        if (!data.game_result && isHumanMode()) {
            document.getElementById('status').textContent = 'Thinking...';
            setTimeout(aiMove, 100);
        }
    }
}

async function aiMove() {
    if (boardState && boardState.game_result) return;
    const mode = document.getElementById('mode-select').value;
    let depth = 0;
    if (mode === 'human_vs_ai' || mode === 'ai_vs_ai') {
        depth = getAiDepth();
    }
    // Time limit: 30s for depth <= 3, 60s for depth 4-5
    const timeLimit = depth >= 4 ? 60000 : 30000;
    document.getElementById('status').textContent = depth > 0
        ? `Searching depth ${depth}...` : 'Thinking...';
    const res = await fetch('/api/ai-move', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({depth, time_limit: timeLimit})
    });
    const data = await res.json();
    if (data.ok && data.last_move) {
        lastMove = data.last_move;
        selectedSq = null;
        legalMoves = [];
        renderBoard(data);
    }
}

async function newGame() {
    const mode = document.getElementById('mode-select').value;
    const color = parseInt(document.getElementById('color-select').value);
    stopAuto();
    lastMove = null;
    selectedSq = null;
    legalMoves = [];
    const res = await fetch('/api/new-game', {
        method: 'POST',
        headers: {'Content-Type': 'application/json'},
        body: JSON.stringify({mode, human_color: color})
    });
    await fetchState();

    // If human plays White, trigger opponent's first move
    if (isHumanMode() && color === 1) {
        setTimeout(aiMove, 300);
    }
}

async function undoMove() {
    await fetch('/api/undo', {method: 'POST'});
    lastMove = null;
    selectedSq = null;
    legalMoves = [];
    await fetchState();
}

function downloadRecord() {
    window.location.href = '/api/record';
}

function toggleAuto() {
    if (autoPlay) {
        stopAuto();
    } else {
        startAuto();
    }
}

function startAuto() {
    autoPlay = true;
    document.getElementById('auto-btn').classList.add('active');
    document.getElementById('auto-btn').textContent = 'Stop';
    autoTick();
}

function stopAuto() {
    autoPlay = false;
    if (autoTimer) clearTimeout(autoTimer);
    autoTimer = null;
    document.getElementById('auto-btn').classList.remove('active');
    document.getElementById('auto-btn').textContent = 'Auto Play';
}

async function autoTick() {
    if (!autoPlay) return;
    if (boardState && boardState.game_result) { stopAuto(); return; }

    const mode = document.getElementById('mode-select').value;
    if (mode === 'random_vs_random' || mode === 'ai_vs_ai') {
        await aiMove();
    } else if (mode === 'human_vs_random' || mode === 'human_vs_ai') {
        // Only auto-move if it's AI's turn
        if (boardState && boardState.side_to_move !== humanColor) {
            await aiMove();
        }
    }

    const speed = parseInt(document.getElementById('speed').value);
    document.getElementById('speed-label').textContent = speed + 'ms';
    autoTimer = setTimeout(autoTick, speed);
}

// Speed slider update
document.getElementById('speed').addEventListener('input', () => {
    document.getElementById('speed-label').textContent =
        document.getElementById('speed').value + 'ms';
});

// Initialize
buildBoard();
fetchState();
</script>

</body>
</html>
"""


# ============================================================
# Main
# ============================================================
def main():
    port = int(sys.argv[1]) if len(sys.argv) > 1 else 8080
    server = HTTPServer(('0.0.0.0', port), GameHandler)
    server.daemon_threads = True
    print(f"Taikyoku Shogi Web GUI")
    print(f"Open http://localhost:{port} in your browser")
    print(f"Press Ctrl+C to stop")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down...")
        server.shutdown()


if __name__ == '__main__':
    main()
