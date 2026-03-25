"""Piece definitions, movements, promotions, and setup for Taikyoku Shogi."""

BOARD_SIZE = 36

# ============================================================
# Colors
# ============================================================
BLACK = 0
WHITE = 1

# ============================================================
# Directions (from Black's perspective)
# Row 0 = top (White's back), Row 35 = bottom (Black's back)
# Black forward = decreasing row
# ============================================================
N, NE, E, SE, S, SW, W, NW = range(8)
ALL_DIRS = list(range(8))
ORTHO = [N, E, S, W]
DIAG = [NE, SE, SW, NW]

# Direction deltas (dr, dc) for Black
DELTAS = [
    (-1, 0),   # N  (forward)
    (-1, 1),   # NE (forward-right)
    (0, 1),    # E  (right)
    (1, 1),    # SE (backward-right)
    (1, 0),    # S  (backward)
    (1, -1),   # SW (backward-left)
    (0, -1),   # W  (left)
    (-1, -1),  # NW (forward-left)
]

def opposite_dir(d):
    return (d + 4) % 8

def get_deltas(d, color):
    """Get (dr, dc) for direction d, adjusted for color."""
    dr, dc = DELTAS[d]
    if color == WHITE:
        dr, dc = -dr, -dc
    return dr, dc

# ============================================================
# Direction classification helpers (for Betza parser)
# ============================================================
FORWARD_DIRS = {N, NE, NW}
BACKWARD_DIRS = {S, SE, SW}
LEFT_DIRS = {W, NW, SW}
RIGHT_DIRS = {E, NE, SE}
SIDEWAYS_DIRS = {E, W}
VERTICAL_DIRS = {N, S}
ORTHO_SET = {N, E, S, W}
DIAG_SET = {NE, SE, SW, NW}
ALL_DIRS_SET = ORTHO_SET | DIAG_SET

# ============================================================
# Betza Notation Parser
# ============================================================
ATOM_BASE = {
    'W': (ORTHO_SET, 1),
    'F': (DIAG_SET, 1),
    'K': (ALL_DIRS_SET, 1),
    'R': (ORTHO_SET, 0),  # 0 = unlimited
    'B': (DIAG_SET, 0),
    'Q': (ALL_DIRS_SET, 0),
}

JUMP_BASE = {
    'D': [(-2, 0), (0, 2), (2, 0), (0, -2)],
    'A': [(-2, 2), (2, 2), (2, -2), (-2, -2)],
    'N': [(-2, 1), (-2, -1), (-1, 2), (-1, -2), (1, 2), (1, -2), (2, 1), (2, -1)],
    'H': [(-3, 0), (0, 3), (3, 0), (0, -3)],
    'G': [(-3, 3), (3, 3), (3, -3), (-3, -3)],
}


def _get_mod_dirs(mod):
    """Convert modifier string to set of matching directions."""
    if not mod:
        return ALL_DIRS_SET
    dirs = set()
    i = 0
    while i < len(mod):
        if i + 1 < len(mod) and mod[i:i+2] in ('fl', 'fr', 'bl', 'br'):
            pair = mod[i:i+2]
            dirs.add({'fl': NW, 'fr': NE, 'bl': SW, 'br': SE}[pair])
            i += 2
        else:
            c = mod[i]
            if c == 'f': dirs |= FORWARD_DIRS
            elif c == 'b': dirs |= BACKWARD_DIRS
            elif c == 'l': dirs |= LEFT_DIRS
            elif c == 'r': dirs |= RIGHT_DIRS
            elif c == 's': dirs |= SIDEWAYS_DIRS
            elif c == 'v': dirs |= VERTICAL_DIRS
            i += 1
    return dirs


def _jump_matches(dr, dc, mod, atom):
    """Check if jump (dr,dc) matches directional modifier for given atom."""
    if not mod:
        return True
    if atom == 'N':
        adr, adc = abs(dr), abs(dc)
        narrow = adr > adc
        if mod == 'ff': return dr < 0 and narrow
        if mod == 'bb': return dr > 0 and narrow
        if mod == 'fs': return dr < 0 and not narrow
        if mod == 'bs': return dr > 0 and not narrow
        if mod == 'v': return narrow
        if mod == 's': return not narrow
    # General: check if direction components match
    for c in mod:
        if c == 'f' and dr < 0: return True
        if c == 'b' and dr > 0: return True
        if c == 'l' and dc < 0: return True
        if c == 'r' and dc > 0: return True
        if c == 's' and dr == 0: return True
        if c == 'v' and dc == 0: return True
    return False


def parse_betza(notation):
    """Parse Betza notation into movement rules.

    Returns dict with:
      'slides': [(direction, max_range), ...]  -- 0 means unlimited
      'jumps': [(delta_row, delta_col), ...]
    Returns None for notations that need manual handling.
    """
    if notation is None:
        return None
    # Skip complex notations with hooks/lions/range-capture
    if '(' in notation or notation.startswith('RANGE') or notation.startswith('HOOK') \
       or notation.startswith('LION') or notation.startswith('AREA'):
        return None

    slides = []
    jumps = []
    i = 0
    n = len(notation)

    while i < n:
        # Collect lowercase modifier
        mod = ''
        while i < n and notation[i].islower():
            mod += notation[i]
            i += 1
        if i >= n:
            break

        atom = notation[i]
        i += 1

        # Collect digit suffix
        rng_str = ''
        while i < n and notation[i].isdigit():
            rng_str += notation[i]
            i += 1
        rng_val = int(rng_str) if rng_str else None

        if atom in ATOM_BASE:
            base_dirs, base_range = ATOM_BASE[atom]
            eff_range = rng_val if rng_val is not None else base_range
            mod_dirs = _get_mod_dirs(mod)
            for d in sorted(base_dirs & mod_dirs):
                slides.append((d, eff_range))
        elif atom in JUMP_BASE:
            for dr, dc in JUMP_BASE[atom]:
                if _jump_matches(dr, dc, mod, atom):
                    jumps.append((dr, dc))
        # Skip unrecognized atoms

    return {'slides': slides, 'jumps': jumps}


# ============================================================
# Piece Data Table
# Each entry: (abbreviation, name, betza_notation_or_None, promotes_to_abbrev)
# betza=None means movement is defined manually (hooks, lions, range capture, etc.)
# promotes_to=None means the piece does not promote.
# ============================================================
PIECE_DEFS = [
    # ---- Base pieces (appear in initial setup) ----
    # Abbreviation, Name, Betza, Promotes_to
    ('AB', 'Angry Boar', 'sK', '+FB'),
    ('B', 'Angle Mover', 'B', 'DH'),
    ('BA', 'Running Bear', None, '+FBR'),       # complex: KfrBblB -> manual
    ('BB', 'Blind Bear', 'sK', '+FSG'),
    ('BC', 'Beast Cadet', 'B2fsR2', 'BO'),
    ('BD', 'Buddhist Devil', 'fB3bsW', '+HT'),
    ('BE', 'Bear Soldier', 'bWsR2fQ', '+SBR'),
    ('BG', 'Angle General', None, '+RDM'),      # range capture diagonal
    ('BI', 'Blind Dog', 'fFbsW', 'VS'),
    ('BL', 'Blue Dragon', 'sR2vRfrB', '+DDR'),
    ('BM', 'Blind Monkey', 'sK', '+FSG'),
    ('BN', 'Cannon Soldier', 'fB5bWsR3fR7', '+CNG'),
    ('BO', 'Beast Officer', 'B3sR2fR3', '+BBR'),
    ('BS', 'Boar Soldier', 'bWsR2fQ', '+RBO'),
    ('BT', 'Blind Tiger', 'FbWsW', '+FSG'),
    ('C', 'Copper General', 'vWfF', 'SM'),
    ('CA', 'Capricorn', None, 'HM'),            # hook diagonal
    ('CC', 'Huai Chicken', 'fFbsW', '+WST'),
    ('CD', 'Ceramic Dove', 'R2B', None),
    ('CE', 'Cloud Eagle', 'sWfF3vWW', '+SEA'),  # custom: 3-square limited diag fwd + step
    ('CG', 'Chicken General', 'bFfR4', '+FCK'),
    ('CH', 'Chariot Soldier', 'sR2vQ', '+HTK'),
    ('CI', 'Stone Chariot', 'fFsR2vR', '+WHR'),
    ('CK', 'Flying Chicken', 'sWfF', '+RHK'),
    ('CL', 'Cloud Dragon', 'BWbR', 'GD'),
    ('CM', 'Climbing Monkey', 'vWfF', 'VS'),
    ('CN', 'Center Standard', 'B3R', 'SD'),
    ('CO', 'Fowl Officer', 'B3fsR2', '+FOW'),
    ('CP', 'Crown Prince', 'K', 'K'),
    ('CR', 'Copper Chariot', 'fB3vR', '+CEL'),
    ('CS', 'Cat Sword', 'F', 'DH'),
    ('CT', 'Fowl Cadet', 'B3fsR3', 'CO'),
    ('D', 'Dog', 'fWfF', '+MUG'),
    ('DE', 'Drunken Elephant', 'fsK', 'CP'),
    ('DG', 'Roaring Dog', None, 'LD'),           # complex jump + slide
    ('DH', 'Dragon Horse', 'WB', 'HF'),
    ('DK', 'Dragon King', 'RF', 'EL'),
    ('DL', 'Howling Dog (Left)', 'bWfR', '+LDG'),
    ('DM', 'Fire Demon', 'vR2sQ', '+FFR'),
    ('DO', 'Donkey', 'R2', 'CD'),
    ('DR', 'Howling Dog (Right)', 'bWfR', '+RDG'),
    ('DS', 'Dark Spirit', 'WfrFbF', '+BSP'),
    ('DV', 'Deva', 'WflFbF', '+KTE'),
    ('EA', 'Earth General', 'vW', 'WE'),
    ('EB', 'Enchanted Badger', 'R2', 'CD'),
    ('EC', 'Earth Chariot', 'WvR', '+RBI'),
    ('ED', 'Earth Dragon', 'fFbWfR2bB', None),   # custom limited
    ('EG', 'Fierce Eagle', 'B2fsW', 'EL'),
    ('EL', 'Flying Eagle', None, '+GEA'),        # complex: QfA
    ('ES', 'Eastern Barbarian', 'sWvW2fF', 'LN'),
    ('EW', 'Evil Wolf', 'fFfsW', '+PWO'),
    ('F', 'Fire General', 'fFvR3', 'GG'),
    ('FC', 'Flying Cat', None, 'R'),              # complex: bWbFfsHfG
    ('FD', 'Flying Dragon', 'A', 'DK'),
    ('FE', 'Free Eagle', None, None),             # complex: QDAHGf(4,4)...
    ('FG', 'Fragrant Elephant', 'Q2', '+EKI'),
    ('FH', 'Flying Horse', 'B2', 'Q'),
    ('FI', 'Fire Dragon', 'RbB2fB4', 'KM'),
    ('FL', 'Fierce Leopard', 'vK', 'B'),
    ('FO', 'Forest Demon', 'fsR3fBbR', '+THR'),
    ('FP', 'Free Pup', 'FsR2vRfB', '+FDG'),
    ('FR', 'Free Demon', 'BvR5sR', 'Q'),
    ('FS', 'Flying Swallow', 'bWfB', 'R'),
    ('FT', 'Free Dream-eater', 'vQsR5', 'Q'),
    ('FY', 'Flying Goose', 'vWfF', 'SW'),
    ('G', 'Gold General', 'WfF', 'R'),
    ('GB', 'Go-between', 'vW', 'DE'),
    ('GC', 'Gold Chariot', 'FsR2vR', '+PPR'),
    ('GD', 'Great Dragon', 'BvR3', '+ADR'),
    ('GE', 'Great Standard', 'RbB3fB', None),
    ('GG', 'Great General', None, None),          # range capture all
    ('GL', 'Gold Stag', 'fBbB2', 'WH'),
    ('GM', 'Great Master', None, None),           # complex: fGfHbB5sR5fBvR
    ('GN', 'Wood General', 'fB2', 'WE'),
    ('GO', 'Gold Bird', None, '+FBI'),            # complex: bF3sW3vF(f(mpFF)3-FF)
    ('GR', 'Great Dove', 'R3B', 'WO'),
    ('GS', 'Great Stag', None, '+FST'),           # complex: RfAbB2
    ('GT', 'Great Turtle', None, '+SPT'),         # complex: jump + slide
    ('GU', 'Guardian of the Gods', 'R3', '+HT'),
    ('H', 'Horse General', 'fFbWfR3', '+FHO'),
    ('HE', "Ram's-head Soldier", 'bWfB', '+TSO'),
    ('HF', 'Horned Hawk', None, '+GHK'),          # complex: QfD
    ('HM', 'Hook Mover', None, None),             # hook orthogonal
    ('HO', 'Horseman', 'sR2vRfB', '+CVL'),
    ('HR', 'Running Horse', None, 'FR'),           # complex: bWbAfQ
    ('HS', 'Horse Soldier', 'bWsR3fQ', 'HR'),
    ('I', 'Iron General', 'fWfF', 'WE'),
    ('K', 'King', 'Q2', None),
    ('KM', 'Kirin Master', None, None),           # complex: vQvHsR3
    ('KR', 'Kirin', 'FvWsD', 'GO'),
    ('L', 'Incense Chariot', 'fR', 'WH'),
    ('LB', 'Longbow Soldier', 'bWsR2fRfB5', '+LBG'),
    ('LC', 'Left Chariot', 'lWfRfrBblB', '+LIC'),
    ('LD', 'Lion Dog', None, '+GEL'),             # complex: QHG
    ('LE', 'Left Dragon', 'lR2rQ', 'VI'),
    ('LG', 'Left General', 'rFvWrW', '+LAR'),
    ('LI', 'Lion Hawk', None, None),              # complex: WBDAN(cK-bK)
    ('LL', 'Little Turtle', None, '+TRT'),        # complex: vQsR2vD
    ('LN', 'Lion', None, '+FFI'),                 # complex: KDAN(cK-bK)
    ('LO', 'Tengu', None, None),                  # hook diagonal
    ('LP', 'Leopard Soldier', 'bWsR2fRfB3', '+RLE'),
    ('LS', 'Little Standard', 'RfB2bF', 'RS'),
    ('LT', 'Left Tiger', 'FrQ', 'TS'),
    ('M', 'Mountain General', 'fB3vW', '+MTA'),
    ('MF', 'Mountain Hawk', None, 'HF'),          # complex: fDbB2RfB
    ('MK', 'Side Monkey', 'bWfFsR', 'SL'),
    ('ML', 'Left Mountain Eagle', None, 'EL'),    # complex: RlAfBblBbrB2
    ('MR', 'Right Mountain Eagle', None, 'EL'),   # complex: RrAblB2fBbrB
    ('MS', 'Mountain Stag', 'fB3fWsR2bR4', 'GS'),
    ('MT', 'Center Master', None, None),          # complex: sR3bB3vDfAvRfB
    ('N', 'Cassia Horse', 'ffN', 'SL'),
    ('NB', 'Northern Barbarian', 'vWsW2fF', 'WO'),
    ('NK', 'Neighboring King', 'fsK', 'SD'),
    ('NT', 'Fierce Wolf', 'FfW', '+BEY'),
    ('O', 'Ox General', 'fFbWfR3', '+FOX'),
    ('OC', 'Ox Chariot', 'WvR', '+POX'),          # same move as EC but different promotion
    ('OK', 'Old Kite', 'B2sW', 'LO'),
    ('OM', 'Old Monkey', 'FbW', '+MWI'),
    ('OR', 'Old Rat', 'fWbF', '+MBI'),
    ('OS', 'Ox Soldier', 'bWsR3fQ', '+ROX'),
    ('OW', 'Swooping Owl', 'fWbF', 'CE'),
    ('OX', 'Flying Ox', 'BvR', '+FOI'),
    ('P', 'Pawn', 'fW', 'G'),
    ('PC', 'Peacock', None, 'LO'),                # hook: bB2fB(fB-sB)
    ('PE', 'Peng Master', None, None),            # complex: fGbB5fBsR5vR
    ('PG', 'Pup General', 'fFbWfR3', 'FP'),      # same as H but different promo
    ('PH', 'Phoenix', 'WA', 'GO'),
    ('PI', 'Pig General', 'bR2fB4', '+FPI'),
    ('PM', 'Phoenix Master', None, None),         # complex: vQfGsR3
    ('PR', 'Prancing Stag', 'B2sW', 'SQ'),       # same as OK but different promo
    ('PS', 'Poisonous Serpent', 'sfW2bW1fF', 'HM'),  # custom limited
    ('Q', 'Free King', 'Q', 'GG'),
    ('R', 'Flying Chariot', 'R', 'DK'),
    ('RA', 'Rain Dragon', 'WbsRFbB', 'GD'),
    ('RB', 'Rushing Bird', 'FsWfW2', 'FR'),
    ('RC', 'Right Chariot', 'rWfRflBbrB', '+RIC'),
    ('RD', 'Reclining Dragon', 'W', 'GD'),
    ('RE', 'River General', 'fFbWfR3', '+HRI'),   # same move as H
    ('RG', 'Right General', 'lFvWlW', '+RAR'),
    ('RH', 'Running Chariot', 'WfBvR', '+CCH'),   # custom: cannon chariot
    ('RI', 'Right Dragon', 'rR2lQ', 'BL'),
    ('RN', 'Running Stag', 'bR2sRfB', '+FST'),
    ('RO', 'Flying General', None, '+FCR'),       # range capture orthogonal
    ('RP', 'Running Pup', 'bWfRblB', '+FLE'),     # custom
    ('RR', 'Running Rabbit', 'bKfQ', 'TF'),
    ('RS', 'Rear Standard', 'B2R', 'CN'),
    ('RT', 'Running Tiger', 'bWfRbrB', '+FTI'),   # custom
    ('RU', 'Running Serpent', 'bWfRblB', '+FSE'),  # same move as RP, different promo
    ('RV', 'Reverse Chariot', 'vR', 'W'),
    ('RW', 'Running Wolf', 'fWfBsR', '+FWO'),
    ('S', 'Silver General', 'FfW', 'VM'),
    ('SA', 'Side Boar', 'KsR', '+FBI'),
    ('SC', 'Crossbow Soldier', 'fB3bWsR3fR5', '+CBG'),
    ('SD', 'Front Standard', 'B3R', 'GE'),        # same as CN but diff promo chain
    ('SE', 'Sword Soldier', 'bWfF', '+SWG'),
    ('SF', 'Side Flyer', 'FsR', 'SI'),
    ('SG', 'Stone General', 'fF', 'WE'),
    ('SI', 'Side Dragon', 'fsR', '+RDR'),
    ('SL', 'Side Soldier', 'bWfR2sR', 'WB'),      # custom limited
    ('SM', 'Side Mover', 'WsR', '+FBI'),
    ('SN', 'Coiled Serpent', 'vWbF', '+CDR'),
    ('SO', 'Soldier', 'WfBvR', '+CVL'),            # custom
    ('SP', 'Spear Soldier', 'WfR', '+SPG'),
    ('SQ', 'Square Mover', 'R', '+SCH'),           # same as R move but diff promo
    ('SR', 'Silver Rabbit', 'fB2bB', 'W'),
    ('SS', 'Side Serpent', 'bWfR3sR', '+GSH'),
    ('ST', 'Strutting Crow', 'fWbF', '+FHK'),
    ('SU', 'Southern Barbarian', 'sWvW2fF', 'GO'),  # same as ES but diff promo
    ('SV', 'Silver Chariot', 'bFfB2vR', '+GWI'),
    ('SW', "Swallow's Wings", 'WsR', '+GSW'),      # same as SM but diff promo
    ('SX', 'Side Ox', 'frFblFsR', 'OX'),
    ('T', 'Tile General', 'bWfF', 'WE'),
    ('TC', 'Tile Chariot', 'frFblFvR', '+RTI'),    # custom
    ('TD', 'Turtle Dove', 'fB5bsW', 'GR'),
    ('TF', 'Treacherous Fox', None, '+MCR'),       # complex: BAGvRvDvH...
    ('TG', 'Fierce Tiger', 'bWfR2fB', '+GTI'),     # custom
    ('TS', 'Turtle-snake', 'KfrBblB', '+DTU'),     # custom
    ('TT', 'Right Tiger', 'FlQ', 'WT'),
    ('VB', 'Fierce Bear', 'fB2fsW', '+GBR'),
    ('VD', 'Fierce Dragon', None, 'GD'),           # range capture + step: W2((cBcdB)-B)
    ('VE', 'Vertical Bear', 'bWsR2fR', '+FBR'),
    ('VG', 'Vice General', None, 'GG'),            # range capture: D((cBcdB)-B)
    ('VH', 'Vertical Horse', 'fFbWfR', 'DH'),
    ('VI', 'Vermillion Sparrow', 'KflBbrB', '+DSP'),  # custom
    ('VL', 'Vertical Leopard', 'WfFfR', '+GLE'),
    ('VM', 'Vertical Mover', 'WvR', 'OX'),         # same as EC but diff promo
    ('VO', 'Fierce Ox', 'vWfB', 'OX'),
    ('VP', 'Vertical Pup', 'bFbWfR', '+LKI'),
    ('VR', 'Vertical Soldier', 'bWfR2sR', 'CH'),   # custom limited
    ('VS', 'Fierce Stag', 'FfW', '+RUB'),
    ('VT', 'Vertical Tiger', 'fRbR2', '+FTI'),
    ('VW', 'Vertical Wolf', 'sWbR3fR', 'RW'),
    ('W', 'Whale', 'BbR', '+GWH'),
    ('WA', 'Water Dragon', 'RfB2bB4', 'PM'),
    ('WB', 'Water Ox', 'FsR2vR', '+GDE'),          # same as GC but diff promo
    ('WC', 'Wood Chariot', 'flFbrFvR', '+WST2'),
    ('WD', 'Wind Dragon', 'FfBbrBsR', '+FDR'),
    ('WE', 'White Elephant', 'Q2', '+EKI'),
    ('WF', 'Side Wolf', 'flFbrFsR', '+FWO'),
    ('WG', 'Water General', 'fB3vW', 'VG'),
    ('WH', 'White Foal', 'vRfB', '+GFO'),
    ('WI', 'Wind Horse', 'fFbR2fR', '+HHO'),
    ('WL', 'Woodland Demon', 'sR2bB2vRfB', '+SPE'),
    ('WN', 'Wind General', 'sfW2bW1fF', '+FWI'),   # uses same betza as PS
    ('WO', 'Wooden Dove', None, None),              # complex: R2BG(G-B2)
    ('WR', 'Sumo Wrestler', 'B3', '+HT'),
    ('WS', 'Western Barbarian', 'sWvW2fF', 'LD'),  # same as ES but diff promo
    ('WT', 'White Tiger', 'vR2sRflB', '+DTI'),
    ('YA', 'Nature Spirit', 'fFbWsR3', '+HT'),

    # ---- Promoted-only pieces ----
    ('+ADR', 'Ancient Dragon', None, None),       # complex: B(v(mppR3)-R)
    ('+BBR', 'Beast Bird', 'BbR2sR3fR', None),
    ('+BEY', "Bear's Eyes", 'K', None),           # same as Crown Prince
    ('+BSP', 'Buddhist Spirit', None, None),      # complex: QDAN(cK-bK)
    ('+CBG', 'Crossbow General', 'bR2sR3fRfB5', None),
    ('+CCH', 'Cannon Chariot', 'WfBvR', None),
    ('+CDR', 'Coiled Dragon', 'vRbB', None),
    ('+CEL', 'Copper Elephant', 'FvR', None),     # step diag + slide vert
    ('+CNG', 'Cannon General', 'bR2sR3fQ', None),
    ('+CVL', 'Cavalier', 'RfB', None),
    ('+DDR', 'Divine Dragon', 'lR2vrRfrB', None), # custom
    ('+DSP', 'Divine Sparrow', 'WfrFflBbB', None),
    ('+DTI', 'Divine Tiger', 'bR2fsRflB', None),
    ('+DTU', 'Divine Turtle', 'KfrBbB', None),
    ('+EKI', 'Elephant King', 'R2B', None),       # same as CD
    ('+FBR', 'Free Bear', 'KfrBbB', None),        # same move as DTU? -- actually different
    ('+FBI', 'Free Boar', 'WfsRfB', None),
    ('+FCK', 'Free Chicken', 'fFsWfR', None),     # custom
    ('+FCR', 'Flying Crocodile', None, None),     # range capture: bB2fB3((cRcdR)-R)
    ('+FDG', 'Free Dog', 'FsR2vRfB', None),      # same as FP
    ('+FDR', 'Free Dragon', 'BbsR', None),
    ('+FFI', 'Furious Fiend', None, None),        # complex: Q3DAN(cK-bK)
    ('+FFR', 'Free Fire', 'sQvR5', None),
    ('+FHK', 'Flying Hawk', 'BfW', None),
    ('+FHO', 'Free Horse', 'fBfsR', None),
    ('+FLE', 'Free Leopard', 'fBsbR', None),
    ('+FOW', 'Fowl', 'B3fsR2', None),            # same as CO
    ('+FOX', 'Free Ox', 'fBfsR', None),           # same as FHO
    ('+FOI', 'Fire Ox', 'vQW', None),
    ('+FPI', 'Free Pig', 'fBfsR', None),          # same as FHO
    ('+FSE', 'Free Serpent', 'vRbB', None),       # same as CDR
    ('+FSG', 'Flying Stag', 'KvR', None),
    ('+FST', 'Free Stag', 'Q', None),             # same as Free King
    ('+FTI', 'Free Tiger', 'fRbR2', None),        # NOTE: overridden to full movement
    ('+FWI', 'Fierce Wind', 'fFbR2fR', None),     # custom
    ('+FWO', 'Free Wolf', 'fBfsR', None),         # same as FHO
    ('+GBR', 'Great Bear', 'WfQ', None),
    ('+GDE', 'Great Dream-eater', 'QsH', None),   # complex: QsH
    ('+GEA', 'Great Eagle', None, None),          # complex: Q(fA-B)
    ('+GEL', 'Great Elephant', None, None),       # complex: fB3(pR-R)(b(mpB)3-B)
    ('+GFO', 'Great Foal', 'vRfB', None),         # same as WH
    ('+GHK', 'Great Hawk', None, None),           # complex: QfD(fD-R)
    ('+GSH', 'Great Shark', 'RbB2fB5', None),
    ('+GSW', 'Gliding Swallow', 'R', None),       # same as Flying Chariot
    ('+GTI', 'Great Tiger', 'WbsR', None),
    ('+GLE', 'Great Leopard', 'bWsR2fRfB3', None),
    ('+GWH', 'Great Whale', 'BbR', None),         # same as W (Whale)... wait
    ('+GWI', 'Goose Wing', 'FsW3vWW', None),     # custom
    ('+HHO', 'Heavenly Horse', 'vNfR', None),     # custom jump
    ('+HRI', 'Huai River', 'WsQ', None),
    ('+HT', 'Heavenly Tetrarch', 'Q4', None),
    ('+HTK', 'Heavenly Tetrarch King', None, None), # complex: QDA(D-R)(A-B)
    ('+KTE', 'King of Teachings', None, None),    # complex: HG(H-R)(G-B)
    ('+LAR', 'Left Army', 'KlQ', None),
    ('+LBG', 'Longbow General', 'fBvRsR5', None),
    ('+LDG', 'Left Dog', 'bWfRbrB', None),
    ('+LIC', 'Left Iron Chariot', 'lWbBfrB', None),
    ('+LKI', 'Leopard King', 'Q5', None),
    ('+MBI', 'Mocking Bird', 'vRfB', None),       # same as WH
    ('+MCR', 'Mountain Crane', None, None),       # complex: QDAHG(4,0)(4,4)...
    ('+MTA', 'Mount Tai', 'BfsR5', None),
    ('+MUG', 'Multi General', 'vRfB', None),      # same as WH
    ('+MWI', 'Mountain Witch', 'BbR', None),      # same as W
    ('+POX', 'Plodding Ox', 'FvR', None),         # same as CEL
    ('+PPR', 'Playful Parrot', 'bB2fB3sR5vR', None),
    ('+PWO', 'Poisonous Wolf', 'K', None),        # same as Crown Prince
    ('+RAR', 'Right Army', 'KrQ', None),
    ('+RBI', 'Reed Bird', 'sR2bB2vR', None),
    ('+RBO', 'Running Boar', 'KsR', None),        # same as SA
    ('+RDG', 'Right Dog', 'bWfRblB', None),       # note: mirror of LDG
    ('+RDM', 'Rain Demon', None, None),           # complex: fR3sR2bB2bR(fmB-B)
    ('+RDR', 'Running Dragon', 'fsQbR5', None),
    ('+RHK', 'Raiding Hawk', 'fFsWfR', None),
    ('+RIC', 'Right Iron Chariot', 'rWbBflB', None),
    ('+RLE', 'Running Leopard', 'bR2sR3fR', None),
    ('+ROX', 'Running Ox', 'bR2fsRfB', None),
    ('+RTI', 'Running Tile', 'vRsR2', None),
    ('+RUB', 'Rushing Boar', 'fsK', None),        # same as DE
    ('+SBR', 'Strong Bear', 'bR2fsQ', None),
    ('+SCH', 'Strong Chariot', 'RfB', None),      # same as CVL
    ('+SEA', 'Strong Eagle', 'Q', None),          # same as Free King
    ('+SPE', 'Stone Peng', 'BsR5', None),         # same as MTA
    ('+SPG', 'Spear General', 'bR2sR3fR', None),  # same as RLE
    ('+SPT', 'Spirit Turtle', None, None),        # complex: QH
    ('+SWG', 'Sword General', 'bWfQ3', None),
    ('+THR', 'Thunder Runner', 'bsR4fQ', None),
    ('+TRT', 'Treasure Turtle', None, None),      # complex: QD
    ('+TSO', 'Tiger Soldier', 'bWfR2fB', None),
    ('+WHR', 'Walking Heron', 'sR2fB2vR', None),  # custom
    ('+WST', 'Wizard Stork', 'fBsbR', None),      # same as FLE
    ('+WST2', 'Wind Snapping Turtle', 'fB2vR', None),
    ('+FBI', 'Free Bird', None, None),            # complex: RbB3(f(mpB)3-B)
]

# ============================================================
# Build lookup dicts from PIECE_DEFS
# ============================================================
PIECE_NAME = {}       # abbrev -> name
PROMOTES_TO = {}      # abbrev -> promoted abbrev (or None)
BETZA = {}            # abbrev -> betza string (or None)
_ALL_ABBREVS = set()

for abbrev, name, betza, promo in PIECE_DEFS:
    PIECE_NAME[abbrev] = name
    BETZA[abbrev] = betza
    if promo is not None:
        PROMOTES_TO[abbrev] = promo
    _ALL_ABBREVS.add(abbrev)

ABBREV_TO_NAME = PIECE_NAME
NAME_TO_ABBREV = {v: k for k, v in PIECE_NAME.items()}

# ============================================================
# Movement Rules
# For each piece abbreviation, stores:
#   'slides': [(direction, max_range), ...]  0=unlimited
#   'jumps':  [(delta_row, delta_col), ...]
#   'hook': 'orth' | 'diag' | None
#   'area': 0 | 2 | 3   (lion move range)
#   'range_capture': [direction, ...]
#   'igui': bool
# ============================================================
def _make_move(slides=None, jumps=None, hook=None, area=0,
               range_capture=None, igui=False):
    return {
        'slides': slides or [],
        'jumps': jumps or [],
        'hook': hook,
        'area': area,
        'range_capture': range_capture or [],
        'igui': igui,
    }


MOVEMENTS = {}

# Auto-parse Betza notations
for abbrev, name, betza, promo in PIECE_DEFS:
    if betza is not None:
        parsed = parse_betza(betza)
        if parsed is not None:
            MOVEMENTS[abbrev] = _make_move(slides=parsed['slides'],
                                           jumps=parsed['jumps'])

# ============================================================
# Manual movement overrides for complex pieces
# ============================================================

# Hook movers
MOVEMENTS['HM'] = _make_move(
    slides=[(N, 0), (E, 0), (S, 0), (W, 0)],
    hook='orth'
)
MOVEMENTS['LO'] = _make_move(  # Tengu
    slides=[(NE, 0), (SE, 0), (SW, 0), (NW, 0)],
    hook='diag'
)
MOVEMENTS['CA'] = _make_move(  # Capricorn
    slides=[(N, 1), (E, 1), (S, 1), (W, 1),
            (NE, 0), (SE, 0), (SW, 0), (NW, 0)],
    hook='diag'
)
MOVEMENTS['PC'] = _make_move(  # Peacock - hook on forward diagonals + limited back diag
    slides=[(SE, 2), (SW, 2), (NE, 0), (NW, 0)],
    hook='diag'  # hook only on forward diagonals
)

# Lion / Area movers
MOVEMENTS['LN'] = _make_move(  # Lion
    slides=[(d, 1) for d in ALL_DIRS],
    jumps=[(-2, 0), (-2, 1), (-2, 2), (-1, 2), (0, 2), (1, 2), (2, 2),
           (2, 1), (2, 0), (2, -1), (2, -2), (1, -2), (0, -2), (-1, -2),
           (-2, -2), (-2, -1)],
    area=2, igui=True
)
MOVEMENTS['LI'] = _make_move(  # Lion Hawk
    slides=[(d, 1) for d in [N, E, S, W]] + [(d, 0) for d in [NE, SE, SW, NW]],
    jumps=[(-2, 0), (-2, 1), (-2, 2), (-1, 2), (0, 2), (1, 2), (2, 2),
           (2, 1), (2, 0), (2, -1), (2, -2), (1, -2), (0, -2), (-1, -2),
           (-2, -2), (-2, -1)],
    area=2, igui=True
)
MOVEMENTS['+FFI'] = _make_move(  # Furious Fiend
    slides=[(d, 3) for d in ALL_DIRS],
    jumps=[(-2, 0), (-2, 1), (-2, 2), (-1, 2), (0, 2), (1, 2), (2, 2),
           (2, 1), (2, 0), (2, -1), (2, -2), (1, -2), (0, -2), (-1, -2),
           (-2, -2), (-2, -1)],
    area=2, igui=True
)
MOVEMENTS['+BSP'] = _make_move(  # Buddhist Spirit
    slides=[(d, 0) for d in ALL_DIRS],
    jumps=[(-2, 0), (-2, 1), (-2, 2), (-1, 2), (0, 2), (1, 2), (2, 2),
           (2, 1), (2, 0), (2, -1), (2, -2), (1, -2), (0, -2), (-1, -2),
           (-2, -2), (-2, -1)],
    area=2, igui=True
)

# Range capturing pieces
MOVEMENTS['GG'] = _make_move(  # Great General
    range_capture=ALL_DIRS
)
MOVEMENTS['VG'] = _make_move(  # Vice General
    jumps=[(-2, 0), (0, 2), (2, 0), (0, -2)],  # Dabbaba jumps
    range_capture=[NE, SE, SW, NW]  # range capture on diagonals
)
MOVEMENTS['RO'] = _make_move(  # Flying General
    range_capture=[N, E, S, W]  # range capture on orthogonals
)
MOVEMENTS['BG'] = _make_move(  # Angle General
    range_capture=[NE, SE, SW, NW]  # range capture on diagonals
)
MOVEMENTS['VD'] = _make_move(  # Fierce Dragon
    slides=[(d, 2) for d in [N, E, S, W]],  # step up to 2 orthogonal
    range_capture=[NE, SE, SW, NW]  # range capture on diagonals
)
MOVEMENTS['+FCR'] = _make_move(  # Flying Crocodile
    slides=[(SE, 2), (SW, 2), (NE, 3), (NW, 3)],
    range_capture=[N, E, S, W]  # range capture on orthogonals
)

# Complex jump-slide pieces
MOVEMENTS['HR'] = _make_move(  # Running Horse: bWbAfQ
    slides=[(S, 1)] + [(d, 0) for d in [N, NE, NW]],
    jumps=[(2, 2), (2, -2)]  # backward alfil (from Black's perspective backward=+row)
)

MOVEMENTS['FC'] = _make_move(  # Flying Cat: bWbFfsHfG
    slides=[(S, 1), (SE, 1), (SW, 1)],  # bW + bF = step backward orth + diag
    jumps=[(-3, 0), (0, 3), (0, -3),    # fsH: forward + sideways threeleaper
           (-3, 3), (-3, -3)]            # fG: forward tripper
)

MOVEMENTS['MF'] = _make_move(  # Mountain Hawk: fDbB2RfB
    slides=[(N, 0), (E, 0), (S, 0), (W, 0),  # R (rook)
            (NE, 0), (NW, 0),                  # fB (forward bishop)
            (SE, 2), (SW, 2)],                  # bB2
    jumps=[(-2, 0)]  # fD: forward dabbaba
)

MOVEMENTS['ML'] = _make_move(  # Left Mountain Eagle: RlAfBblBbrB2
    slides=[(N, 0), (E, 0), (S, 0), (W, 0),  # R
            (NE, 0), (NW, 0),                  # fB
            (SW, 0),                            # blB
            (SE, 2)],                           # brB2
    jumps=[(-2, -2)]  # lA: left alfil
)

MOVEMENTS['MR'] = _make_move(  # Right Mountain Eagle: RrAblB2fBbrB
    slides=[(N, 0), (E, 0), (S, 0), (W, 0),  # R
            (NE, 0), (NW, 0),                  # fB
            (SE, 0),                            # brB
            (SW, 2)],                           # blB2
    jumps=[(-2, 2)]  # rA: right alfil
)

MOVEMENTS['GS'] = _make_move(  # Great Stag: RfAbB2
    slides=[(N, 0), (E, 0), (S, 0), (W, 0),  # R
            (SE, 2), (SW, 2)],                  # bB2
    jumps=[(-2, 2), (-2, -2)]  # fA: forward alfil
)

MOVEMENTS['DG'] = _make_move(  # Roaring Dog: HfGRbB3fB
    slides=[(N, 0), (E, 0), (S, 0), (W, 0),  # R
            (NE, 0), (NW, 0),                  # fB
            (SE, 3), (SW, 3)],                  # bB3
    jumps=[(-3, 0), (0, 3), (0, -3), (3, 0),  # H (threeleaper)
           (-3, 3), (-3, -3)]                   # fG (forward tripper)
)

MOVEMENTS['LD'] = _make_move(  # Lion Dog: QHG
    slides=[(d, 0) for d in ALL_DIRS],  # Q
    jumps=[(-3, 0), (0, 3), (3, 0), (0, -3),    # H
           (-3, 3), (3, 3), (3, -3), (-3, -3)]   # G
)

MOVEMENTS['EL'] = _make_move(  # Flying Eagle: QfA
    slides=[(d, 0) for d in ALL_DIRS],  # Q
    jumps=[(-2, 2), (-2, -2)]  # fA
)

MOVEMENTS['HF'] = _make_move(  # Horned Hawk: QfD
    slides=[(d, 0) for d in ALL_DIRS],  # Q
    jumps=[(-2, 0)]  # fD
)

MOVEMENTS['FE'] = _make_move(  # Free Eagle: Q + all jumps
    slides=[(d, 0) for d in ALL_DIRS],
    jumps=[(-2, 0), (0, 2), (2, 0), (0, -2),    # D
           (-2, 2), (2, 2), (2, -2), (-2, -2),   # A
           (-3, 0), (0, 3), (3, 0), (0, -3),     # H
           (-3, 3), (3, 3), (3, -3), (-3, -3),    # G
           (-4, 4)],                                # f(4,4) approx
    igui=True
)

MOVEMENTS['WO'] = _make_move(  # Wooden Dove: R2BG(G-B2)
    slides=[(d, 2) for d in ORTHO] + [(d, 0) for d in DIAG],
    jumps=[(-3, 3), (3, 3), (3, -3), (-3, -3)]  # G tripper
)

MOVEMENTS['TF'] = _make_move(  # Treacherous Fox
    slides=[(d, 0) for d in ALL_DIRS],  # Q-like
    jumps=[(-2, 0), (0, 2), (2, 0), (0, -2),     # D
           (-2, 2), (2, 2), (2, -2), (-2, -2),    # A
           (-3, 0), (0, 3), (3, 0), (0, -3)]      # H
)

MOVEMENTS['KM'] = _make_move(  # Kirin Master: vQvHsR3
    slides=[(N, 0), (S, 0),                # vQ (vertical queen = unlimited N/S)
            (NE, 0), (NW, 0), (SE, 0), (SW, 0),  # vQ diag part
            (E, 3), (W, 3)],               # sR3
    jumps=[(-3, 0), (3, 0)]                # vH
)

MOVEMENTS['PM'] = _make_move(  # Phoenix Master: vQfGsR3
    slides=[(N, 0), (S, 0),
            (NE, 0), (NW, 0), (SE, 0), (SW, 0),
            (E, 3), (W, 3)],
    jumps=[(-3, 3), (-3, -3)]              # fG
)

MOVEMENTS['GM'] = _make_move(  # Great Master: fGfHbB5sR5fBvR
    slides=[(NE, 0), (NW, 0),              # fB
            (N, 0), (S, 0),                 # vR
            (SE, 5), (SW, 5),               # bB5
            (E, 5), (W, 5)],               # sR5
    jumps=[(-3, 3), (-3, -3),              # fG
           (-3, 0)]                         # fH
)

MOVEMENTS['MT'] = _make_move(  # Center Master: sR3bB3vDfAvRfB
    slides=[(N, 0), (S, 0),                # vR
            (NE, 0), (NW, 0),              # fB
            (E, 3), (W, 3),                # sR3
            (SE, 3), (SW, 3)],             # bB3
    jumps=[(-2, 0), (2, 0),                # vD
           (-2, 2), (-2, -2)]              # fA
)

MOVEMENTS['PE'] = _make_move(  # Peng Master: fGbB5fBsR5vR
    slides=[(N, 0), (S, 0),                # vR
            (NE, 0), (NW, 0),              # fB
            (SE, 5), (SW, 5),              # bB5
            (E, 5), (W, 5)],              # sR5
    jumps=[(-3, 3), (-3, -3)]             # fG
)

MOVEMENTS['LL'] = _make_move(  # Little Turtle: vQsR2vD
    slides=[(d, 0) for d in [N, S, NE, NW, SE, SW]] +
           [(E, 2), (W, 2)],
    jumps=[(-2, 0), (2, 0)]  # vD
)

MOVEMENTS['GT'] = _make_move(  # Great Turtle: similar to Little Turtle but stronger
    slides=[(d, 0) for d in ALL_DIRS] + [(E, 2), (W, 2)],
    jumps=[(-2, 0), (2, 0)]
)

MOVEMENTS['+SPT'] = _make_move(  # Spirit Turtle: QH
    slides=[(d, 0) for d in ALL_DIRS],
    jumps=[(-3, 0), (0, 3), (3, 0), (0, -3)]
)

MOVEMENTS['+TRT'] = _make_move(  # Treasure Turtle: QD
    slides=[(d, 0) for d in ALL_DIRS],
    jumps=[(-2, 0), (0, 2), (2, 0), (0, -2)]
)

MOVEMENTS['+HTK'] = _make_move(  # Heavenly Tetrarch King: QDA(D-R)(A-B)
    slides=[(d, 0) for d in ALL_DIRS],
    jumps=[(-2, 0), (0, 2), (2, 0), (0, -2),
           (-2, 2), (2, 2), (2, -2), (-2, -2)]
)

MOVEMENTS['+KTE'] = _make_move(  # King of Teachings: HG(H-R)(G-B)
    slides=[],  # Complex jump-slide
    jumps=[(-3, 0), (0, 3), (3, 0), (0, -3),
           (-3, 3), (3, 3), (3, -3), (-3, -3)]
)
# King of Teachings can also range after jumping - approximate as Q + jumps
MOVEMENTS['+KTE'] = _make_move(
    slides=[(d, 0) for d in ALL_DIRS],
    jumps=[(-3, 0), (0, 3), (3, 0), (0, -3),
           (-3, 3), (3, 3), (3, -3), (-3, -3)]
)

MOVEMENTS['+GEA'] = _make_move(  # Great Eagle: Q(fA-B) = Q + forward alfil
    slides=[(d, 0) for d in ALL_DIRS],
    jumps=[(-2, 2), (-2, -2)]
)

MOVEMENTS['+GHK'] = _make_move(  # Great Hawk: QfD(fD-R)
    slides=[(d, 0) for d in ALL_DIRS],
    jumps=[(-2, 0)]
)

MOVEMENTS['+GEL'] = _make_move(  # Great Elephant: approx as Q with some jumps
    slides=[(NE, 3), (NW, 3)] + [(d, 0) for d in ORTHO],
    jumps=[]
)

MOVEMENTS['+ADR'] = _make_move(  # Ancient Dragon: B(v(mppR3)-R)
    slides=[(d, 0) for d in DIAG] + [(N, 0), (S, 0)],
    jumps=[]
)

MOVEMENTS['+MCR'] = _make_move(  # Mountain Crane: approx Q + all jumps
    slides=[(d, 0) for d in ALL_DIRS],
    jumps=[(-2, 0), (0, 2), (2, 0), (0, -2),
           (-2, 2), (2, 2), (2, -2), (-2, -2),
           (-3, 0), (0, 3), (3, 0), (0, -3),
           (-3, 3), (3, 3), (3, -3), (-3, -3)]
)

MOVEMENTS['+RDM'] = _make_move(  # Rain Demon: complex, approx
    slides=[(N, 3), (E, 2), (W, 2), (S, 0),
            (SE, 2), (SW, 2)],
    jumps=[]
)

MOVEMENTS['GO'] = _make_move(  # Gold Bird: complex, approx
    slides=[(SE, 3), (SW, 3), (N, 0), (S, 0)],
    jumps=[]
)

MOVEMENTS['+FBI'] = _make_move(  # Free Bird: complex, approx
    slides=[(d, 0) for d in ORTHO] + [(SE, 3), (SW, 3)],
    jumps=[]
)

MOVEMENTS['+GDE'] = _make_move(  # Great Dream-eater: QsH
    slides=[(d, 0) for d in ALL_DIRS],
    jumps=[(0, 3), (0, -3)]  # sH
)

MOVEMENTS['+HHO'] = _make_move(  # Heavenly Horse: vNfR
    slides=[(N, 0)],  # fR
    jumps=[(-2, 1), (-2, -1), (2, 1), (2, -1)]  # vN (narrow forward+backward)
)

MOVEMENTS['BA'] = _make_move(  # Running Bear: KfrBblB -> actually manual notation
    slides=[(d, 1) for d in ALL_DIRS] +
           [(NE, 0), (SW, 0)]  # frB + blB: unlimited forward-right and backward-left diag
)

MOVEMENTS['ED'] = _make_move(  # Earth Dragon: fFbWfR2bB
    slides=[(NE, 1), (NW, 1),      # fF (step forward diag)
            (S, 1),                  # bW (step backward)
            (N, 2),                  # fR2 (limited 2 forward orth... actually fR2 might be different)
            (SE, 0), (SW, 0)]       # bB (unlimited backward diag)
)

MOVEMENTS['RH'] = _make_move(  # Running Chariot: WfBvR -> actually the parsed betza is fine
    slides=[(N, 1), (E, 1), (S, 1), (W, 1),  # W
            (NE, 0), (NW, 0),                  # fB
            (N, 0), (S, 0)]                    # vR - override the W step for N,S
)
# Fix: merge slides - unlimited overrides limited for same direction
def _merge_slides(slides):
    """Merge slide rules, keeping the longest range for each direction."""
    best = {}
    for d, r in slides:
        if d not in best or (r == 0) or (best[d] != 0 and r > best[d]):
            best[d] = r
    return [(d, r) for d, r in sorted(best.items())]

# Apply merge to all movements
for abbrev in MOVEMENTS:
    m = MOVEMENTS[abbrev]
    if m['slides']:
        m['slides'] = _merge_slides(m['slides'])


# ============================================================
# Ensure all piece types have movement entries
# For any piece missing movement, create a dummy (no moves)
# ============================================================
for abbrev, name, betza, promo in PIECE_DEFS:
    if abbrev not in MOVEMENTS:
        MOVEMENTS[abbrev] = _make_move()


# ============================================================
# Piece Rankings (for range capture restrictions)
# ============================================================
RANK_ROYAL = 0       # King, Crown Prince - cannot be range-captured
RANK_GREAT = 1       # Great General
RANK_VICE = 2        # Vice General
RANK_RANGE_CAP = 3   # Flying General, Angle General, Fierce Dragon, Flying Crocodile
RANK_NORMAL = 4      # All other pieces

PIECE_RANK = {}
for abbrev in _ALL_ABBREVS:
    PIECE_RANK[abbrev] = RANK_NORMAL

PIECE_RANK['K'] = RANK_ROYAL
PIECE_RANK['CP'] = RANK_ROYAL
PIECE_RANK['+BEY'] = RANK_ROYAL   # Bear's Eyes promotes from Crown Prince -> still royal? No.
PIECE_RANK['GG'] = RANK_GREAT
PIECE_RANK['VG'] = RANK_VICE
PIECE_RANK['RO'] = RANK_RANGE_CAP
PIECE_RANK['BG'] = RANK_RANGE_CAP
PIECE_RANK['VD'] = RANK_RANGE_CAP
PIECE_RANK['+FCR'] = RANK_RANGE_CAP


# ============================================================
# Royal pieces (capturing all of opponent's royals = win)
# ============================================================
ROYAL_PIECES = {'K', 'CP'}


# ============================================================
# Piece Values (rough material values for evaluation)
# ============================================================
PIECE_VALUE = {}
_VALUE_MAP = {
    # Royal
    'K': 100000, 'CP': 50000,
    # Generals
    'GG': 20000, 'VG': 15000, 'Q': 10000,
    # Range capturers
    'RO': 12000, 'BG': 12000,
    # Strong pieces
    'FE': 9000, 'LI': 9000, 'LN': 8000, 'LD': 8000,
    'DM': 7000, 'HF': 7000, 'EL': 7000,
    'DH': 6000, 'DK': 6000, 'HM': 5000,
    'R': 5000, 'B': 4500,
    'FR': 6000, 'FT': 6000,
    # Medium pieces
    'G': 3000, 'S': 2800, 'C': 2500,
    'VM': 3000, 'SM': 3000,
    # Weak pieces
    'P': 500, 'D': 600, 'GB': 400,
    'I': 1000, 'SG': 800, 'T': 900, 'EA': 700,
    'L': 2000, 'N': 1500,
}

for abbrev in _ALL_ABBREVS:
    if abbrev in _VALUE_MAP:
        PIECE_VALUE[abbrev] = _VALUE_MAP[abbrev]
    elif abbrev.startswith('+'):
        PIECE_VALUE[abbrev] = 5000  # promoted pieces are generally strong
    else:
        # Estimate from movement count
        m = MOVEMENTS.get(abbrev, _make_move())
        val = len(m['slides']) * 400 + len(m['jumps']) * 300
        if m['hook']: val += 2000
        if m['area']: val += 3000
        if m['range_capture']: val += 5000
        PIECE_VALUE[abbrev] = max(val, 500)


# ============================================================
# Initial Board Setup
# Each row is a list of 36 abbreviation strings (or '' for empty)
# Listed from Black's rank 1 (row 35) up to rank 12 (row 24)
# ============================================================
_SETUP_RANKS = [
    # Rank 1 (Black's back rank - King's row) -> absolute row 35
    'L  TS RR W  DM ML LO BC HR FR ED CD FT Q  RS LG G  K  CP G  RG RS Q  FT WO ED FR HR BC LO MR DM W  RR WT L',
    # Rank 2 -> row 34
    'RV WE TD FS CO RA FO MS RP RU SS GR RT BA BD WR S  NK DE S  GU YA BA RT GR SS RU RP MS FO RA CO FS TD FG RV',
    # Rank 3 -> row 33
    'GC SI RN RW BG RO LT LE BO WD FP RB OK PC WA FI C  KM PM C  FI WA PC OK RB FP WD BO RI TT RO BG RW RN SI GC',
    # Rank 4 -> row 32
    'SV VE N  PI CG PG H  O  CN SA SR GL LN CT GS VD WL GG VG WL VD GS CT LN GL SR SA CN O  H  PG CG PI N  VE SV',
    # Rank 5 -> row 31
    'CI CE B  R  WF FC MF VT SO LS CL CR RH HE VO GD GO DV DS GO GD VO HE RH CR CL LS SO VT MF FC WF R  B  CE CI',
    # Rank 6 -> row 30
    'WC WH DL SM PR WB FL EG FD PS FY ST BI WG F  KR CA GT LL HM PH F  WG BI ST FY PS FD EG FL WB PR SM DR WH WC',
    # Rank 7 -> row 29
    'TC VW SX DO FH VB AB EW WI CK OM CC WS ES VS NT TF PE MT TF NT VS SU NB CC OM CK WI EW AB VB FH DO SX VW TC',
    # Rank 8 -> row 28
    'EC BL EB HO OW CM CS SW BM BT OC SF BB OR SQ SN RD LI FE RD SN SQ OR BB SF OC BT BM SW CS CM OW HO EB VI EC',
    # Rank 9 -> row 27
    'CH SL VR WN RE M  SD HS GN OS EA BS SG LP T  BE I  GM GE I  BE T  LP SG BS EA OS GN HS SD M  RE WN VR SL CH',
    # Rank 10 -> row 26
    'LC MK VM OX LB VP VH BN DH DK SE HF EL SP VL TG SC LD DG SC TG VL SP EL HF SE DK DH BN VH VP LB OX VM MK RC',
    # Rank 11 (pawn row) -> row 25
    'P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P',
    # Rank 12 (sparse: dogs and go-betweens) -> row 24
    '.  .  .  .  .  D  .  .  .  .  GB .  .  .  .  D  .  .  .  .  .  .  .  D  .  .  .  .  GB .  .  .  .  D  .  .',
]

def _parse_setup_row(row_str):
    """Parse a setup row string into list of 36 piece abbreviations."""
    parts = row_str.split()
    assert len(parts) == 36, f"Expected 36 pieces, got {len(parts)}: {parts}"
    return [p if p != '.' else '' for p in parts]

SETUP_BLACK = [_parse_setup_row(r) for r in _SETUP_RANKS]

# White's setup is the 180-degree rotation of Black's
# Same rank order as Black (rank 1 = back rank first), but files reversed
SETUP_WHITE = []
for rank in SETUP_BLACK:
    SETUP_WHITE.append(list(reversed(rank)))


def get_initial_board():
    """Return the initial 36x36 board as a 2D list.

    board[row][col] = (piece_abbrev, color) or None
    Row 0 = top (White's back rank)
    Row 35 = bottom (Black's back rank)
    """
    board = [[None] * BOARD_SIZE for _ in range(BOARD_SIZE)]

    # Place White's pieces (rows 0-11)
    for rank_idx, rank_pieces in enumerate(SETUP_WHITE):
        row = rank_idx  # White's rank 1 at row 0
        for col, piece in enumerate(rank_pieces):
            if piece:
                board[row][col] = (piece, WHITE)

    # Place Black's pieces (rows 24-35)
    for rank_idx, rank_pieces in enumerate(SETUP_BLACK):
        row = 35 - rank_idx  # Black's rank 1 at row 35
        for col, piece in enumerate(rank_pieces):
            if piece:
                board[row][col] = (piece, BLACK)

    return board
