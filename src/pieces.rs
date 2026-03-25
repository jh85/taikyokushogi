/// Piece definitions: 301 piece types, Betza parser, movements, promotions, setup.
use crate::types::*;
use std::sync::OnceLock;
use std::collections::HashMap;

// ============================================================
// Piece type IDs (1-based, 0 = no piece)
// ============================================================
// Generated from PIECE_DEFS order below. ID = index + 1.

struct PieceDef {
    abbrev: &'static str,
    name: &'static str,
    betza: Option<&'static str>,
    promotes_to: Option<&'static str>, // abbrev of promoted form
    value: i32,
    rank: u8,
}

// All piece definitions. Order determines piece type ID (1-based).
const PIECE_DEFS: &[PieceDef] = &[
    // ---- Base pieces (appear in initial setup) ----
    PieceDef { abbrev: "AB", name: "Angry Boar", betza: Some("sK"), promotes_to: Some("+FB"), value: 1600, rank: RANK_NORMAL },
    PieceDef { abbrev: "B", name: "Angle Mover", betza: Some("B"), promotes_to: Some("DH"), value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "BA", name: "Running Bear", betza: None, promotes_to: Some("+FBR"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "BB", name: "Blind Bear", betza: Some("sK"), promotes_to: Some("+FSG"), value: 1600, rank: RANK_NORMAL },
    PieceDef { abbrev: "BC", name: "Beast Cadet", betza: Some("B2fsR2"), promotes_to: Some("BO"), value: 2800, rank: RANK_NORMAL },
    PieceDef { abbrev: "BD", name: "Buddhist Devil", betza: Some("fB3bsW"), promotes_to: Some("+HT"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "BE", name: "Bear Soldier", betza: Some("bWsR2fQ"), promotes_to: Some("+SBR"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "BG", name: "Angle General", betza: None, promotes_to: Some("+RDM"), value: 12000, rank: RANK_RANGE_CAP },
    PieceDef { abbrev: "BI", name: "Blind Dog", betza: Some("fFbsW"), promotes_to: Some("VS"), value: 1400, rank: RANK_NORMAL },
    PieceDef { abbrev: "BL", name: "Blue Dragon", betza: Some("sR2vRfrB"), promotes_to: Some("+DDR"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "BM", name: "Blind Monkey", betza: Some("sK"), promotes_to: Some("+FSG"), value: 1600, rank: RANK_NORMAL },
    PieceDef { abbrev: "BN", name: "Cannon Soldier", betza: Some("fB5bWsR3fR7"), promotes_to: Some("+CNG"), value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "BO", name: "Beast Officer", betza: Some("B3sR2fR3"), promotes_to: Some("+BBR"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "BS", name: "Boar Soldier", betza: Some("bWsR2fQ"), promotes_to: Some("+RBO"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "BT", name: "Blind Tiger", betza: Some("FbWsW"), promotes_to: Some("+FSG"), value: 1800, rank: RANK_NORMAL },
    PieceDef { abbrev: "C", name: "Copper General", betza: Some("vWfF"), promotes_to: Some("SM"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "CA", name: "Capricorn", betza: None, promotes_to: Some("HM"), value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "CC", name: "Huai Chicken", betza: Some("fFbsW"), promotes_to: Some("+WST"), value: 1400, rank: RANK_NORMAL },
    PieceDef { abbrev: "CD", name: "Ceramic Dove", betza: Some("R2B"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "CE", name: "Cloud Eagle", betza: Some("sWfF3vWW"), promotes_to: Some("+SEA"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "CG", name: "Chicken General", betza: Some("bFfR4"), promotes_to: Some("+FCK"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "CH", name: "Chariot Soldier", betza: Some("sR2vQ"), promotes_to: Some("+HTK"), value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "CI", name: "Stone Chariot", betza: Some("fFsR2vR"), promotes_to: Some("+WHR"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "CK", name: "Flying Chicken", betza: Some("sWfF"), promotes_to: Some("+RHK"), value: 1200, rank: RANK_NORMAL },
    PieceDef { abbrev: "CL", name: "Cloud Dragon", betza: Some("BWbR"), promotes_to: Some("GD"), value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "CM", name: "Climbing Monkey", betza: Some("vWfF"), promotes_to: Some("VS"), value: 1500, rank: RANK_NORMAL },
    PieceDef { abbrev: "CN", name: "Center Standard", betza: Some("B3R"), promotes_to: Some("SD"), value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "CO", name: "Fowl Officer", betza: Some("B3fsR2"), promotes_to: Some("+FOW"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "CP", name: "Crown Prince", betza: Some("K"), promotes_to: Some("K"), value: 50000, rank: RANK_ROYAL },
    PieceDef { abbrev: "CR", name: "Copper Chariot", betza: Some("fB3vR"), promotes_to: Some("+CEL"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "CS", name: "Cat Sword", betza: Some("F"), promotes_to: Some("DH"), value: 1200, rank: RANK_NORMAL },
    PieceDef { abbrev: "CT", name: "Fowl Cadet", betza: Some("B3fsR3"), promotes_to: Some("CO"), value: 3200, rank: RANK_NORMAL },
    PieceDef { abbrev: "D", name: "Dog", betza: Some("fWfF"), promotes_to: Some("+MUG"), value: 600, rank: RANK_NORMAL },
    PieceDef { abbrev: "DE", name: "Drunken Elephant", betza: Some("fsK"), promotes_to: Some("CP"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "DG", name: "Roaring Dog", betza: None, promotes_to: Some("LD"), value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "DH", name: "Dragon Horse", betza: Some("WB"), promotes_to: Some("HF"), value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "DK", name: "Dragon King", betza: Some("RF"), promotes_to: Some("EL"), value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "DL", name: "Howling Dog (Left)", betza: Some("bWfR"), promotes_to: Some("+LDG"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "DM", name: "Fire Demon", betza: Some("vR2sQ"), promotes_to: Some("+FFR"), value: 7000, rank: RANK_NORMAL },
    PieceDef { abbrev: "DO", name: "Donkey", betza: Some("R2"), promotes_to: Some("CD"), value: 1500, rank: RANK_NORMAL },
    PieceDef { abbrev: "DR", name: "Howling Dog (Right)", betza: Some("bWfR"), promotes_to: Some("+RDG"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "DS", name: "Dark Spirit", betza: Some("WfrFbF"), promotes_to: Some("+BSP"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "DV", name: "Deva", betza: Some("WflFbF"), promotes_to: Some("+KTE"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "EA", name: "Earth General", betza: Some("vW"), promotes_to: Some("WE"), value: 700, rank: RANK_NORMAL },
    PieceDef { abbrev: "EB", name: "Enchanted Badger", betza: Some("R2"), promotes_to: Some("CD"), value: 1500, rank: RANK_NORMAL },
    PieceDef { abbrev: "EC", name: "Earth Chariot", betza: Some("WvR"), promotes_to: Some("+RBI"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "ED", name: "Earth Dragon", betza: Some("fFbWfR2bB"), promotes_to: None, value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "EG", name: "Fierce Eagle", betza: Some("B2fsW"), promotes_to: Some("EL"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "EL", name: "Flying Eagle", betza: None, promotes_to: Some("+GEA"), value: 7000, rank: RANK_NORMAL },
    PieceDef { abbrev: "ES", name: "Eastern Barbarian", betza: Some("sWvW2fF"), promotes_to: Some("LN"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "EW", name: "Evil Wolf", betza: Some("fFfsW"), promotes_to: Some("+PWO"), value: 1400, rank: RANK_NORMAL },
    PieceDef { abbrev: "F", name: "Fire General", betza: Some("fFvR3"), promotes_to: Some("GG"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "FC", name: "Flying Cat", betza: None, promotes_to: Some("R"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "FD", name: "Flying Dragon", betza: Some("A"), promotes_to: Some("DK"), value: 1500, rank: RANK_NORMAL },
    PieceDef { abbrev: "FE", name: "Free Eagle", betza: None, promotes_to: None, value: 9000, rank: RANK_NORMAL },
    PieceDef { abbrev: "FG", name: "Fragrant Elephant", betza: Some("Q2"), promotes_to: Some("+EKI"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "FH", name: "Flying Horse", betza: Some("B2"), promotes_to: Some("Q"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "FI", name: "Fire Dragon", betza: Some("RbB2fB4"), promotes_to: Some("KM"), value: 5500, rank: RANK_NORMAL },
    PieceDef { abbrev: "FL", name: "Fierce Leopard", betza: Some("vK"), promotes_to: Some("B"), value: 1800, rank: RANK_NORMAL },
    PieceDef { abbrev: "FO", name: "Forest Demon", betza: Some("fsR3fBbR"), promotes_to: Some("+THR"), value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "FP", name: "Free Pup", betza: Some("FsR2vRfB"), promotes_to: Some("+FDG"), value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "FR", name: "Free Demon", betza: Some("BvR5sR"), promotes_to: Some("Q"), value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "FS", name: "Flying Swallow", betza: Some("bWfB"), promotes_to: Some("R"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "FT", name: "Free Dream-eater", betza: Some("vQsR5"), promotes_to: Some("Q"), value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "FY", name: "Flying Goose", betza: Some("vWfF"), promotes_to: Some("SW"), value: 1500, rank: RANK_NORMAL },
    PieceDef { abbrev: "G", name: "Gold General", betza: Some("WfF"), promotes_to: Some("R"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "GB", name: "Go-between", betza: Some("vW"), promotes_to: Some("DE"), value: 400, rank: RANK_NORMAL },
    PieceDef { abbrev: "GC", name: "Gold Chariot", betza: Some("FsR2vR"), promotes_to: Some("+PPR"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "GD", name: "Great Dragon", betza: Some("BvR3"), promotes_to: Some("+ADR"), value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "GE", name: "Great Standard", betza: Some("RbB3fB"), promotes_to: None, value: 5500, rank: RANK_NORMAL },
    PieceDef { abbrev: "GG", name: "Great General", betza: None, promotes_to: None, value: 20000, rank: RANK_GREAT },
    PieceDef { abbrev: "GL", name: "Gold Stag", betza: Some("fBbB2"), promotes_to: Some("WH"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "GM", name: "Great Master", betza: None, promotes_to: None, value: 7000, rank: RANK_NORMAL },
    PieceDef { abbrev: "GN", name: "Wood General", betza: Some("fB2"), promotes_to: Some("WE"), value: 1000, rank: RANK_NORMAL },
    PieceDef { abbrev: "GO", name: "Gold Bird", betza: None, promotes_to: Some("+FBI"), value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "GR", name: "Great Dove", betza: Some("R3B"), promotes_to: Some("WO"), value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "GS", name: "Great Stag", betza: None, promotes_to: Some("+FST"), value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "GT", name: "Great Turtle", betza: None, promotes_to: Some("+SPT"), value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "GU", name: "Guardian of the Gods", betza: Some("R3"), promotes_to: Some("+HT"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "H", name: "Horse General", betza: Some("fFbWfR3"), promotes_to: Some("+FHO"), value: 2800, rank: RANK_NORMAL },
    PieceDef { abbrev: "HE", name: "Ram's-head Soldier", betza: Some("bWfB"), promotes_to: Some("+TSO"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "HF", name: "Horned Hawk", betza: None, promotes_to: Some("+GHK"), value: 7000, rank: RANK_NORMAL },
    PieceDef { abbrev: "HM", name: "Hook Mover", betza: None, promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "HO", name: "Horseman", betza: Some("sR2vRfB"), promotes_to: Some("+CVL"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "HR", name: "Running Horse", betza: None, promotes_to: Some("FR"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "HS", name: "Horse Soldier", betza: Some("bWsR3fQ"), promotes_to: Some("HR"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "I", name: "Iron General", betza: Some("fWfF"), promotes_to: Some("WE"), value: 1000, rank: RANK_NORMAL },
    PieceDef { abbrev: "K", name: "King", betza: Some("Q2"), promotes_to: None, value: 100000, rank: RANK_ROYAL },
    PieceDef { abbrev: "KM", name: "Kirin Master", betza: None, promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "KR", name: "Kirin", betza: Some("FvWsD"), promotes_to: Some("GO"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "L", name: "Incense Chariot", betza: Some("fR"), promotes_to: Some("WH"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "LB", name: "Longbow Soldier", betza: Some("bWsR2fRfB5"), promotes_to: Some("+LBG"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "LC", name: "Left Chariot", betza: Some("lWfRfrBblB"), promotes_to: Some("+LIC"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "LD", name: "Lion Dog", betza: None, promotes_to: Some("+GEL"), value: 8000, rank: RANK_NORMAL },
    PieceDef { abbrev: "LE", name: "Left Dragon", betza: Some("lR2rQ"), promotes_to: Some("VI"), value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "LG", name: "Left General", betza: Some("rFvWrW"), promotes_to: Some("+LAR"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "LI", name: "Lion Hawk", betza: None, promotes_to: None, value: 9000, rank: RANK_NORMAL },
    PieceDef { abbrev: "LL", name: "Little Turtle", betza: None, promotes_to: Some("+TRT"), value: 5500, rank: RANK_NORMAL },
    PieceDef { abbrev: "LN", name: "Lion", betza: None, promotes_to: Some("+FFI"), value: 8000, rank: RANK_NORMAL },
    PieceDef { abbrev: "LO", name: "Tengu", betza: None, promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "LP", name: "Leopard Soldier", betza: Some("bWsR2fRfB3"), promotes_to: Some("+RLE"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "LS", name: "Little Standard", betza: Some("RfB2bF"), promotes_to: Some("RS"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "LT", name: "Left Tiger", betza: Some("FrQ"), promotes_to: Some("TS"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "M", name: "Mountain General", betza: Some("fB3vW"), promotes_to: Some("+MTA"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "MF", name: "Mountain Hawk", betza: None, promotes_to: Some("HF"), value: 5500, rank: RANK_NORMAL },
    PieceDef { abbrev: "MK", name: "Side Monkey", betza: Some("bWfFsR"), promotes_to: Some("SL"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "ML", name: "Left Mountain Eagle", betza: None, promotes_to: Some("EL"), value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "MR", name: "Right Mountain Eagle", betza: None, promotes_to: Some("EL"), value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "MS", name: "Mountain Stag", betza: Some("fB3fWsR2bR4"), promotes_to: Some("GS"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "MT", name: "Center Master", betza: None, promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "N", name: "Cassia Horse", betza: Some("ffN"), promotes_to: Some("SL"), value: 1500, rank: RANK_NORMAL },
    PieceDef { abbrev: "NB", name: "Northern Barbarian", betza: Some("vWsW2fF"), promotes_to: Some("WO"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "NK", name: "Neighboring King", betza: Some("fsK"), promotes_to: Some("SD"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "NT", name: "Fierce Wolf", betza: Some("FfW"), promotes_to: Some("+BEY"), value: 1800, rank: RANK_NORMAL },
    PieceDef { abbrev: "O", name: "Ox General", betza: Some("fFbWfR3"), promotes_to: Some("+FOX"), value: 2800, rank: RANK_NORMAL },
    PieceDef { abbrev: "OC", name: "Ox Chariot", betza: Some("fR"), promotes_to: Some("+POX"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "OK", name: "Old Kite", betza: Some("B2sW"), promotes_to: Some("LO"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "OM", name: "Old Monkey", betza: Some("FbW"), promotes_to: Some("+MWI"), value: 1500, rank: RANK_NORMAL },
    PieceDef { abbrev: "OR", name: "Old Rat", betza: Some("fWbF"), promotes_to: Some("+MBI"), value: 1200, rank: RANK_NORMAL },
    PieceDef { abbrev: "OS", name: "Ox Soldier", betza: Some("bWsR3fQ"), promotes_to: Some("+ROX"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "OW", name: "Swooping Owl", betza: Some("fWbF"), promotes_to: Some("CE"), value: 1200, rank: RANK_NORMAL },
    PieceDef { abbrev: "OX", name: "Flying Ox", betza: Some("BvR"), promotes_to: Some("+FOI"), value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "P", name: "Pawn", betza: Some("fW"), promotes_to: Some("G"), value: 500, rank: RANK_NORMAL },
    PieceDef { abbrev: "PC", name: "Peacock", betza: None, promotes_to: Some("LO"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "PE", name: "Peng Master", betza: None, promotes_to: None, value: 7000, rank: RANK_NORMAL },
    PieceDef { abbrev: "PG", name: "Pup General", betza: Some("fFbWfR3"), promotes_to: Some("FP"), value: 2800, rank: RANK_NORMAL },
    PieceDef { abbrev: "PH", name: "Phoenix", betza: Some("WA"), promotes_to: Some("GO"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "PI", name: "Pig General", betza: Some("bR2fB4"), promotes_to: Some("+FPI"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "PM", name: "Phoenix Master", betza: None, promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "PR", name: "Prancing Stag", betza: Some("B2sW"), promotes_to: Some("SQ"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "PS", name: "Poisonous Serpent", betza: Some("sfW2bW1fF"), promotes_to: Some("HM"), value: 1500, rank: RANK_NORMAL },
    PieceDef { abbrev: "Q", name: "Free King", betza: Some("Q"), promotes_to: Some("GG"), value: 10000, rank: RANK_NORMAL },
    PieceDef { abbrev: "R", name: "Flying Chariot", betza: Some("R"), promotes_to: Some("DK"), value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "RA", name: "Rain Dragon", betza: Some("WbsRFbB"), promotes_to: Some("GD"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "RB", name: "Rushing Bird", betza: Some("FsWfW2"), promotes_to: Some("FR"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "RC", name: "Right Chariot", betza: Some("rWfRflBbrB"), promotes_to: Some("+RIC"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "RD", name: "Reclining Dragon", betza: Some("W"), promotes_to: Some("GD"), value: 1200, rank: RANK_NORMAL },
    PieceDef { abbrev: "RE", name: "River General", betza: Some("fFbWfR3"), promotes_to: Some("+HRI"), value: 2800, rank: RANK_NORMAL },
    PieceDef { abbrev: "RG", name: "Right General", betza: Some("lFvWlW"), promotes_to: Some("+RAR"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "RH", name: "Running Chariot", betza: Some("WfBvR"), promotes_to: Some("+CCH"), value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "RI", name: "Right Dragon", betza: Some("rR2lQ"), promotes_to: Some("BL"), value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "RN", name: "Running Stag", betza: Some("bR2sRfB"), promotes_to: Some("+FST"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "RO", name: "Flying General", betza: None, promotes_to: Some("+FCR"), value: 12000, rank: RANK_RANGE_CAP },
    PieceDef { abbrev: "RP", name: "Running Pup", betza: Some("bWfRblB"), promotes_to: Some("+FLE"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "RR", name: "Running Rabbit", betza: Some("bKfQ"), promotes_to: Some("TF"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "RS", name: "Rear Standard", betza: Some("B2R"), promotes_to: Some("CN"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "RT", name: "Running Tiger", betza: Some("bWfRbrB"), promotes_to: Some("+FTI"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "RU", name: "Running Serpent", betza: Some("bWfRblB"), promotes_to: Some("+FSE"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "RV", name: "Reverse Chariot", betza: Some("vR"), promotes_to: Some("W"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "RW", name: "Running Wolf", betza: Some("fWfBsR"), promotes_to: Some("+FWO"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "S", name: "Silver General", betza: Some("FfW"), promotes_to: Some("VM"), value: 2800, rank: RANK_NORMAL },
    PieceDef { abbrev: "SA", name: "Side Boar", betza: Some("KsR"), promotes_to: Some("+FBI"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "SC", name: "Crossbow Soldier", betza: Some("fB3bWsR3fR5"), promotes_to: Some("+CBG"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "SD", name: "Front Standard", betza: Some("B3R"), promotes_to: Some("GE"), value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "SE", name: "Sword Soldier", betza: Some("bWfF"), promotes_to: Some("+SWG"), value: 1200, rank: RANK_NORMAL },
    PieceDef { abbrev: "SF", name: "Side Flyer", betza: Some("FsR"), promotes_to: Some("SI"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "SG", name: "Stone General", betza: Some("fF"), promotes_to: Some("WE"), value: 800, rank: RANK_NORMAL },
    PieceDef { abbrev: "SI", name: "Side Dragon", betza: Some("fsR"), promotes_to: Some("+RDR"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "SL", name: "Side Soldier", betza: Some("bWfR2sR"), promotes_to: Some("WB"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "SM", name: "Side Mover", betza: Some("WsR"), promotes_to: Some("+FBI"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "SN", name: "Coiled Serpent", betza: Some("vWbF"), promotes_to: Some("+CDR"), value: 1500, rank: RANK_NORMAL },
    PieceDef { abbrev: "SO", name: "Soldier", betza: Some("WfBvR"), promotes_to: Some("+CVL"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "SP", name: "Spear Soldier", betza: Some("WfR"), promotes_to: Some("+SPG"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "SQ", name: "Square Mover", betza: Some("R"), promotes_to: Some("+SCH"), value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "SR", name: "Silver Rabbit", betza: Some("fB2bB"), promotes_to: Some("W"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "SS", name: "Side Serpent", betza: Some("bWfR3sR"), promotes_to: Some("+GSH"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "ST", name: "Strutting Crow", betza: Some("fWbF"), promotes_to: Some("+FHK"), value: 1200, rank: RANK_NORMAL },
    PieceDef { abbrev: "SU", name: "Southern Barbarian", betza: Some("sWvW2fF"), promotes_to: Some("GO"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "SV", name: "Silver Chariot", betza: Some("bFfB2vR"), promotes_to: Some("+GWI"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "SW", name: "Swallow's Wings", betza: Some("WsR"), promotes_to: Some("+GSW"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "SX", name: "Side Ox", betza: Some("frFblFsR"), promotes_to: Some("OX"), value: 2800, rank: RANK_NORMAL },
    PieceDef { abbrev: "T", name: "Tile General", betza: Some("bWfF"), promotes_to: Some("WE"), value: 900, rank: RANK_NORMAL },
    PieceDef { abbrev: "TC", name: "Tile Chariot", betza: Some("frFblFvR"), promotes_to: Some("+RTI"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "TD", name: "Turtle Dove", betza: Some("fB5bsW"), promotes_to: Some("GR"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "TF", name: "Treacherous Fox", betza: None, promotes_to: Some("+MCR"), value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "TG", name: "Fierce Tiger", betza: Some("fR"), promotes_to: Some("+GTI"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "TS", name: "Turtle-snake", betza: Some("KfrBblB"), promotes_to: Some("+DTU"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "TT", name: "Right Tiger", betza: Some("FlQ"), promotes_to: Some("WT"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "VB", name: "Fierce Bear", betza: Some("fB2fsW"), promotes_to: Some("+GBR"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "VD", name: "Fierce Dragon", betza: None, promotes_to: Some("GD"), value: 6000, rank: RANK_RANGE_CAP },
    PieceDef { abbrev: "VE", name: "Vertical Bear", betza: Some("bWsR2fR"), promotes_to: Some("+FBR"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "VG", name: "Vice General", betza: None, promotes_to: Some("GG"), value: 15000, rank: RANK_VICE },
    PieceDef { abbrev: "VH", name: "Vertical Horse", betza: Some("fFbWfR"), promotes_to: Some("DH"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "VI", name: "Vermillion Sparrow", betza: Some("KflBbrB"), promotes_to: Some("+DSP"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "VL", name: "Vertical Leopard", betza: Some("WfFfR"), promotes_to: Some("+GLE"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "VM", name: "Vertical Mover", betza: Some("WvR"), promotes_to: Some("OX"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "VO", name: "Fierce Ox", betza: Some("vWfB"), promotes_to: Some("OX"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "VP", name: "Vertical Pup", betza: Some("bFbWfR"), promotes_to: Some("+LKI"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "VR", name: "Vertical Soldier", betza: Some("bWfR2sR"), promotes_to: Some("CH"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "VS", name: "Fierce Stag", betza: Some("FfW"), promotes_to: Some("+RUB"), value: 1800, rank: RANK_NORMAL },
    PieceDef { abbrev: "VT", name: "Vertical Tiger", betza: Some("fRbR2"), promotes_to: Some("+FTI"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "VW", name: "Vertical Wolf", betza: Some("sWbR3fR"), promotes_to: Some("RW"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "W", name: "Whale", betza: Some("BbR"), promotes_to: Some("+GWH"), value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "WA", name: "Water Dragon", betza: Some("RfB2bB4"), promotes_to: Some("PM"), value: 5500, rank: RANK_NORMAL },
    PieceDef { abbrev: "WB", name: "Water Ox", betza: Some("FsR2vR"), promotes_to: Some("+GDE"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "WC", name: "Wood Chariot", betza: Some("flFbrFvR"), promotes_to: Some("+WST2"), value: 3000, rank: RANK_NORMAL },
    PieceDef { abbrev: "WD", name: "Wind Dragon", betza: Some("FfBbrBsR"), promotes_to: Some("+FDR"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "WE", name: "White Elephant", betza: Some("Q2"), promotes_to: Some("+EKI"), value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "WF", name: "Side Wolf", betza: Some("flFbrFsR"), promotes_to: Some("+FWO"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "WG", name: "Water General", betza: Some("fB3vW"), promotes_to: Some("VG"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "WH", name: "White Foal", betza: Some("vRfB"), promotes_to: Some("+GFO"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "WI", name: "Wind Horse", betza: Some("fFbR2fR"), promotes_to: Some("+HHO"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "WL", name: "Woodland Demon", betza: Some("sR2bB2vRfB"), promotes_to: Some("+SPE"), value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "WN", name: "Wind General", betza: Some("sfW2bW1fF"), promotes_to: Some("+FWI"), value: 1500, rank: RANK_NORMAL },
    PieceDef { abbrev: "WO", name: "Wooden Dove", betza: None, promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "WR", name: "Sumo Wrestler", betza: Some("B3"), promotes_to: Some("+HT"), value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "WS", name: "Western Barbarian", betza: Some("sWvW2fF"), promotes_to: Some("LD"), value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "WT", name: "White Tiger", betza: Some("vR2sRflB"), promotes_to: Some("+DTI"), value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "YA", name: "Nature Spirit", betza: Some("fFbWsR3"), promotes_to: Some("+HT"), value: 2500, rank: RANK_NORMAL },
    // ---- Promoted-only pieces ----
    PieceDef { abbrev: "+ADR", name: "Ancient Dragon", betza: None, promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+BBR", name: "Beast Bird", betza: Some("BbR2sR3fR"), promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+BEY", name: "Bear's Eyes", betza: Some("K"), promotes_to: None, value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+BSP", name: "Buddhist Spirit", betza: None, promotes_to: None, value: 9000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+CBG", name: "Crossbow General", betza: Some("bR2sR3fRfB5"), promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+CCH", name: "Cannon Chariot", betza: Some("WfBvR"), promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+CDR", name: "Coiled Dragon", betza: Some("vRbB"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+CEL", name: "Copper Elephant", betza: Some("FvR"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+CNG", name: "Cannon General", betza: Some("bR2sR3fQ"), promotes_to: None, value: 5500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+CVL", name: "Cavalier", betza: Some("RfB"), promotes_to: None, value: 5500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+DDR", name: "Divine Dragon", betza: Some("lR2vrRfrB"), promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+DSP", name: "Divine Sparrow", betza: Some("WfrFflBbB"), promotes_to: None, value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+DTI", name: "Divine Tiger", betza: Some("bR2fsRflB"), promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+DTU", name: "Divine Turtle", betza: Some("KfrBbB"), promotes_to: None, value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+EKI", name: "Elephant King", betza: Some("R2B"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FB", name: "Free Boar", betza: Some("WfsRfB"), promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FBR", name: "Free Bear", betza: Some("WfQ"), promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FBI", name: "Free Bird", betza: None, promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FCK", name: "Free Chicken", betza: Some("fFsWfR"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FCR", name: "Flying Crocodile", betza: None, promotes_to: None, value: 8000, rank: RANK_RANGE_CAP },
    PieceDef { abbrev: "+FDG", name: "Free Dog", betza: Some("FsR2vRfB"), promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FDR", name: "Free Dragon", betza: Some("BbsR"), promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FFI", name: "Furious Fiend", betza: None, promotes_to: None, value: 10000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FFR", name: "Free Fire", betza: Some("sQvR5"), promotes_to: None, value: 8000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FHK", name: "Flying Hawk", betza: Some("BfW"), promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FHO", name: "Free Horse", betza: Some("fBfsR"), promotes_to: None, value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FLE", name: "Free Leopard", betza: Some("fBsbR"), promotes_to: None, value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FOI", name: "Fire Ox", betza: Some("vQW"), promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FOW", name: "Fowl", betza: Some("B3fsR2"), promotes_to: None, value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FOX", name: "Free Ox", betza: Some("fBfsR"), promotes_to: None, value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FPI", name: "Free Pig", betza: Some("fBfsR"), promotes_to: None, value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FSE", name: "Free Serpent", betza: Some("vRbB"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FSG", name: "Flying Stag", betza: Some("KvR"), promotes_to: None, value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FST", name: "Free Stag", betza: Some("Q"), promotes_to: None, value: 10000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FTI", name: "Free Tiger", betza: Some("fRbR2"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FWI", name: "Fierce Wind", betza: Some("fFbR2fR"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+FWO", name: "Free Wolf", betza: Some("fBfsR"), promotes_to: None, value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+GBR", name: "Great Bear", betza: Some("WfQ"), promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+GDE", name: "Great Dream-eater", betza: None, promotes_to: None, value: 8000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+GEA", name: "Great Eagle", betza: None, promotes_to: None, value: 9000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+GEL", name: "Great Elephant", betza: None, promotes_to: None, value: 9000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+GFO", name: "Great Foal", betza: Some("vRfB"), promotes_to: None, value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+GHK", name: "Great Hawk", betza: None, promotes_to: None, value: 8000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+GSH", name: "Great Shark", betza: Some("RbB2fB5"), promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+GSW", name: "Gliding Swallow", betza: Some("R"), promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+GTI", name: "Great Tiger", betza: Some("WbsR"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+GLE", name: "Great Leopard", betza: Some("bWsR2fRfB3"), promotes_to: None, value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+GWH", name: "Great Whale", betza: Some("BbR"), promotes_to: None, value: 5500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+GWI", name: "Goose Wing", betza: Some("FsW3vWW"), promotes_to: None, value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+HHO", name: "Heavenly Horse", betza: None, promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+HRI", name: "Huai River", betza: Some("WsQ"), promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+HT", name: "Heavenly Tetrarch", betza: Some("Q4"), promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+HTK", name: "Heavenly Tetrarch King", betza: None, promotes_to: None, value: 8000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+KTE", name: "King of Teachings", betza: None, promotes_to: None, value: 8000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+LAR", name: "Left Army", betza: Some("KlQ"), promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+LBG", name: "Longbow General", betza: Some("fBvRsR5"), promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+LDG", name: "Left Dog", betza: Some("bWfRbrB"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+LIC", name: "Left Iron Chariot", betza: Some("lWbBfrB"), promotes_to: None, value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+LKI", name: "Leopard King", betza: Some("Q5"), promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+MBI", name: "Mocking Bird", betza: Some("vRfB"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+MCR", name: "Mountain Crane", betza: None, promotes_to: None, value: 9000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+MTA", name: "Mount Tai", betza: Some("BfsR5"), promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+MUG", name: "Multi General", betza: Some("vRfB"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+MWI", name: "Mountain Witch", betza: Some("BbR"), promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+POX", name: "Plodding Ox", betza: Some("FvR"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+PPR", name: "Playful Parrot", betza: Some("bB2fB3sR5vR"), promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+PWO", name: "Poisonous Wolf", betza: Some("K"), promotes_to: None, value: 2000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+RAR", name: "Right Army", betza: Some("KrQ"), promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+RBI", name: "Reed Bird", betza: Some("sR2bB2vR"), promotes_to: None, value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+RBO", name: "Running Boar", betza: Some("KsR"), promotes_to: None, value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+RDG", name: "Right Dog", betza: Some("bWfRblB"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+RDM", name: "Rain Demon", betza: None, promotes_to: None, value: 7000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+RDR", name: "Running Dragon", betza: Some("fsQbR5"), promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+RHK", name: "Raiding Hawk", betza: Some("fFsWfR"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+RIC", name: "Right Iron Chariot", betza: Some("rWbBflB"), promotes_to: None, value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+RLE", name: "Running Leopard", betza: Some("bR2sR3fR"), promotes_to: None, value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+ROX", name: "Running Ox", betza: Some("bR2fsRfB"), promotes_to: None, value: 5000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+RTI", name: "Running Tile", betza: Some("vRsR2"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+RUB", name: "Rushing Boar", betza: Some("fsK"), promotes_to: None, value: 2500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+SBR", name: "Strong Bear", betza: Some("bR2fsQ"), promotes_to: None, value: 5500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+SCH", name: "Strong Chariot", betza: Some("RfB"), promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+SEA", name: "Strong Eagle", betza: Some("Q"), promotes_to: None, value: 10000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+SPE", name: "Stone Peng", betza: Some("BsR5"), promotes_to: None, value: 6000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+SPG", name: "Spear General", betza: Some("bR2sR3fR"), promotes_to: None, value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+SPT", name: "Spirit Turtle", betza: None, promotes_to: None, value: 8000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+SWG", name: "Sword General", betza: Some("bWfQ3"), promotes_to: None, value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+THR", name: "Thunder Runner", betza: Some("bsR4fQ"), promotes_to: None, value: 5500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+TRT", name: "Treasure Turtle", betza: None, promotes_to: None, value: 7000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+TSO", name: "Tiger Soldier", betza: Some("bWfR2fB"), promotes_to: None, value: 3500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+WHR", name: "Walking Heron", betza: Some("sR2fB2vR"), promotes_to: None, value: 4000, rank: RANK_NORMAL },
    PieceDef { abbrev: "+WST", name: "Wizard Stork", betza: Some("fBsbR"), promotes_to: None, value: 4500, rank: RANK_NORMAL },
    PieceDef { abbrev: "+WST2", name: "Wind Snapping Turtle", betza: Some("fB2vR"), promotes_to: None, value: 3500, rank: RANK_NORMAL },
];

// ============================================================
// Piece data store (initialized once)
// ============================================================
pub struct PieceStore {
    pub count: usize,
    abbrev_to_id: HashMap<String, u16>,
    movements: Vec<Movement>,
    promotes_to: Vec<u16>,   // 0 = no promotion
}

static STORE: OnceLock<PieceStore> = OnceLock::new();

pub fn store() -> &'static PieceStore {
    STORE.get_or_init(|| init_store())
}

pub fn abbrev(pt: u16) -> &'static str {
    if pt == 0 || pt as usize > PIECE_DEFS.len() { return ""; }
    PIECE_DEFS[pt as usize - 1].abbrev
}

pub fn name(pt: u16) -> &'static str {
    if pt == 0 || pt as usize > PIECE_DEFS.len() { return ""; }
    PIECE_DEFS[pt as usize - 1].name
}

pub fn value(pt: u16) -> i32 {
    if pt == 0 || pt as usize > PIECE_DEFS.len() { return 0; }
    PIECE_DEFS[pt as usize - 1].value
}

pub fn rank(pt: u16) -> u8 {
    if pt == 0 || pt as usize > PIECE_DEFS.len() { return RANK_NORMAL; }
    PIECE_DEFS[pt as usize - 1].rank
}

pub fn is_royal(pt: u16) -> bool {
    rank(pt) == RANK_ROYAL
}

pub fn promotes_to(pt: u16) -> Option<u16> {
    let s = store();
    if pt == 0 || pt as usize > s.count { return None; }
    let p = s.promotes_to[pt as usize];
    if p == 0 { None } else { Some(p) }
}

pub fn movement(pt: u16) -> &'static Movement {
    let s = store();
    if pt == 0 || pt as usize > s.count {
        static EMPTY: OnceLock<Movement> = OnceLock::new();
        return EMPTY.get_or_init(Movement::empty);
    }
    &s.movements[pt as usize]
}

pub fn find_by_abbrev(ab: &str) -> Option<u16> {
    store().abbrev_to_id.get(ab).copied()
}

pub fn num_piece_types() -> usize {
    PIECE_DEFS.len()
}

/// Pieces that MUST promote when reaching the farthest rank (forward-only pieces).
const FORCED_PROMO_ABBREVS: &[&str] = &[
    "P",   // Pawn
    "SG",  // Stone General
    "I",   // Iron General
    "D",   // Dog
    "GN",  // Wood General
    "L",   // Incense Chariot (Lance)
    "OC",  // Ox Chariot
    "TG",  // Fierce Tiger
];

static FORCED_PROMO_SET: OnceLock<Vec<u16>> = OnceLock::new();

/// Check if this piece type MUST promote at the farthest rank.
pub fn must_promote_at_far_rank(pt: u16) -> bool {
    let set = FORCED_PROMO_SET.get_or_init(|| {
        FORCED_PROMO_ABBREVS.iter()
            .filter_map(|ab| find_by_abbrev(ab))
            .collect()
    });
    set.contains(&pt)
}

// ============================================================
// Betza Parser (same logic as Python version)
// ============================================================
fn parse_betza(notation: &str) -> Option<Movement> {
    if notation.contains('(') { return None; }

    let bytes = notation.as_bytes();
    let n = bytes.len();
    let mut slides = Vec::new();
    let mut jumps = Vec::new();
    let mut i = 0;

    while i < n {
        // Collect lowercase modifier
        let mod_start = i;
        while i < n && bytes[i].is_ascii_lowercase() { i += 1; }
        let modifier = &notation[mod_start..i];
        if i >= n { break; }

        let atom = bytes[i] as char;
        i += 1;

        // Collect digit suffix
        let dig_start = i;
        while i < n && bytes[i].is_ascii_digit() { i += 1; }
        let range_val: Option<u8> = if dig_start < i {
            notation[dig_start..i].parse().ok()
        } else {
            None
        };

        match atom {
            'W' | 'F' | 'K' | 'R' | 'B' | 'Q' => {
                let (base_dirs, base_range) = match atom {
                    'W' => (&[N, E, S, W][..], 1u8),
                    'F' => (&[NE, SE, SW, NW][..], 1),
                    'K' => (&[N, NE, E, SE, S, SW, W, NW][..], 1),
                    'R' => (&[N, E, S, W][..], 0),
                    'B' => (&[NE, SE, SW, NW][..], 0),
                    'Q' => (&[N, NE, E, SE, S, SW, W, NW][..], 0),
                    _ => unreachable!(),
                };
                let eff_range = range_val.unwrap_or(base_range);
                let mod_dirs = get_mod_dirs(modifier);
                for &d in base_dirs {
                    if mod_dirs & (1 << d) != 0 {
                        slides.push((d as u8, eff_range));
                    }
                }
            }
            'D' | 'A' | 'N' | 'H' | 'G' => {
                let base_jumps: &[(i8, i8)] = match atom {
                    'D' => &[(-2,0),(0,2),(2,0),(0,-2)],
                    'A' => &[(-2,2),(2,2),(2,-2),(-2,-2)],
                    'N' => &[(-2,1),(-2,-1),(-1,2),(-1,-2),(1,2),(1,-2),(2,1),(2,-1)],
                    'H' => &[(-3,0),(0,3),(3,0),(0,-3)],
                    'G' => &[(-3,3),(3,3),(3,-3),(-3,-3)],
                    _ => unreachable!(),
                };
                for &(dr, dc) in base_jumps {
                    if jump_matches(dr, dc, modifier, atom) {
                        jumps.push((dr, dc));
                    }
                }
            }
            _ => {}
        }
    }

    // Merge slides: keep longest range per direction
    let mut best = [0u8; 8]; // 0 means not present
    let mut present = [false; 8];
    for &(d, r) in &slides {
        let di = d as usize;
        if !present[di] || r == 0 || (best[di] != 0 && r > best[di]) {
            best[di] = r;
            present[di] = true;
        }
    }
    let merged: Vec<(u8, u8)> = (0..8)
        .filter(|&d| present[d])
        .map(|d| (d as u8, best[d]))
        .collect();

    Some(Movement {
        slides: merged,
        jumps,
        hook: None,
        area: 0,
        range_capture: Vec::new(),
        igui: false,
    })
}

fn get_mod_dirs(modifier: &str) -> u16 {
    if modifier.is_empty() { return 0xFF; } // all 8 directions
    let bytes = modifier.as_bytes();
    let mut dirs: u16 = 0;
    let mut i = 0;
    while i < bytes.len() {
        if i + 1 < bytes.len() {
            let pair = &modifier[i..i+2];
            match pair {
                "fl" => { dirs |= 1 << NW; i += 2; continue; }
                "fr" => { dirs |= 1 << NE; i += 2; continue; }
                "bl" => { dirs |= 1 << SW; i += 2; continue; }
                "br" => { dirs |= 1 << SE; i += 2; continue; }
                _ => {}
            }
        }
        match bytes[i] {
            b'f' => dirs |= (1 << N) | (1 << NE) | (1 << NW),
            b'b' => dirs |= (1 << S) | (1 << SE) | (1 << SW),
            b'l' => dirs |= (1 << W) | (1 << NW) | (1 << SW),
            b'r' => dirs |= (1 << E) | (1 << NE) | (1 << SE),
            b's' => dirs |= (1 << E) | (1 << W),
            b'v' => dirs |= (1 << N) | (1 << S),
            _ => {}
        }
        i += 1;
    }
    dirs
}

fn jump_matches(dr: i8, dc: i8, modifier: &str, atom: char) -> bool {
    if modifier.is_empty() { return true; }
    if atom == 'N' {
        let narrow = dr.unsigned_abs() > dc.unsigned_abs();
        return match modifier {
            "ff" => dr < 0 && narrow,
            "bb" => dr > 0 && narrow,
            "fs" => dr < 0 && !narrow,
            "bs" => dr > 0 && !narrow,
            "f" => dr < 0,
            "b" => dr > 0,
            "v" => narrow,
            "s" => !narrow,
            _ => true,
        };
    }
    for b in modifier.bytes() {
        match b {
            b'f' if dr < 0 => return true,
            b'b' if dr > 0 => return true,
            b'l' if dc < 0 => return true,
            b'r' if dc > 0 => return true,
            b's' if dr == 0 => return true,
            b'v' if dc == 0 => return true,
            _ => {}
        }
    }
    false
}

// ============================================================
// Manual movement overrides for complex pieces
// ============================================================
fn manual_movement(abbrev: &str) -> Option<Movement> {
    let all_dirs_slide: Vec<(u8, u8)> = (0..8).map(|d| (d as u8, 0)).collect();
    let all_dirs_step: Vec<(u8, u8)> = (0..8).map(|d| (d as u8, 1)).collect();
    let lion_jumps: Vec<(i8, i8)> = {
        let mut v = Vec::new();
        for dr in -2i8..=2 { for dc in -2i8..=2 {
            if (dr != 0 || dc != 0) && (dr.abs() > 1 || dc.abs() > 1) {
                v.push((dr, dc));
            }
        }}
        v
    };

    match abbrev {
        "HM" => Some(Movement { slides: vec![(N as u8,0),(E as u8,0),(S as u8,0),(W as u8,0)], jumps: vec![], hook: Some(HookType::Orthogonal), area: 0, range_capture: vec![], igui: false }),
        "LO" => Some(Movement { slides: vec![(NE as u8,0),(SE as u8,0),(SW as u8,0),(NW as u8,0)], jumps: vec![], hook: Some(HookType::Diagonal), area: 0, range_capture: vec![], igui: false }),
        "CA" => Some(Movement { slides: vec![(N as u8,1),(E as u8,1),(S as u8,1),(W as u8,1),(NE as u8,0),(SE as u8,0),(SW as u8,0),(NW as u8,0)], jumps: vec![], hook: Some(HookType::Diagonal), area: 0, range_capture: vec![], igui: false }),
        "PC" => Some(Movement { slides: vec![(SE as u8,2),(SW as u8,2),(NE as u8,0),(NW as u8,0)], jumps: vec![], hook: Some(HookType::Diagonal), area: 0, range_capture: vec![], igui: false }),
        "LN" => Some(Movement { slides: all_dirs_step.clone(), jumps: lion_jumps.clone(), hook: None, area: 2, range_capture: vec![], igui: true }),
        "LI" => Some(Movement { slides: { let mut s = vec![(N as u8,1),(E as u8,1),(S as u8,1),(W as u8,1)]; s.extend([(NE as u8,0),(SE as u8,0),(SW as u8,0),(NW as u8,0)]); s }, jumps: lion_jumps.clone(), hook: None, area: 2, range_capture: vec![], igui: true }),
        "+FFI" => Some(Movement { slides: (0..8).map(|d| (d as u8, 3)).collect(), jumps: lion_jumps.clone(), hook: None, area: 2, range_capture: vec![], igui: true }),
        "+BSP" => Some(Movement { slides: all_dirs_slide.clone(), jumps: lion_jumps.clone(), hook: None, area: 2, range_capture: vec![], igui: true }),
        "GG" => Some(Movement { slides: vec![], jumps: vec![], hook: None, area: 0, range_capture: (0..8).map(|d| d as u8).collect(), igui: false }),
        "VG" => Some(Movement { slides: vec![], jumps: vec![(-2,0),(0,2),(2,0),(0,-2)], hook: None, area: 0, range_capture: vec![NE as u8, SE as u8, SW as u8, NW as u8], igui: false }),
        "RO" => Some(Movement { slides: vec![], jumps: vec![], hook: None, area: 0, range_capture: vec![N as u8, E as u8, S as u8, W as u8], igui: false }),
        "BG" => Some(Movement { slides: vec![], jumps: vec![], hook: None, area: 0, range_capture: vec![NE as u8, SE as u8, SW as u8, NW as u8], igui: false }),
        "VD" => Some(Movement { slides: vec![(N as u8,2),(E as u8,2),(S as u8,2),(W as u8,2)], jumps: vec![], hook: None, area: 0, range_capture: vec![NE as u8, SE as u8, SW as u8, NW as u8], igui: false }),
        "+FCR" => Some(Movement { slides: vec![(SE as u8,2),(SW as u8,2),(NE as u8,3),(NW as u8,3)], jumps: vec![], hook: None, area: 0, range_capture: vec![N as u8, E as u8, S as u8, W as u8], igui: false }),
        "BA" => Some(Movement { slides: { let mut s = all_dirs_step.clone(); s.push((NE as u8, 0)); s.push((SW as u8, 0)); s }, jumps: vec![], hook: None, area: 0, range_capture: vec![], igui: false }),
        "HR" => Some(Movement { slides: vec![(S as u8,1),(N as u8,0),(NE as u8,0),(NW as u8,0)], jumps: vec![(2,2),(2,-2)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "FC" => Some(Movement { slides: vec![(S as u8,1),(SE as u8,1),(SW as u8,1)], jumps: vec![(-3,0),(0,3),(0,-3),(-3,3),(-3,-3)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "EL" => Some(Movement { slides: all_dirs_slide.clone(), jumps: vec![(-2,2),(-2,-2)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "HF" => Some(Movement { slides: all_dirs_slide.clone(), jumps: vec![(-2,0)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "FE" => Some(Movement { slides: all_dirs_slide.clone(), jumps: vec![(-2,0),(0,2),(2,0),(0,-2),(-2,2),(2,2),(2,-2),(-2,-2),(-3,0),(0,3),(3,0),(0,-3),(-3,3),(3,3),(3,-3),(-3,-3)], hook: None, area: 0, range_capture: vec![], igui: true }),
        "WO" => Some(Movement { slides: { let mut s: Vec<(u8,u8)> = vec![(N as u8,2),(E as u8,2),(S as u8,2),(W as u8,2)]; s.extend([(NE as u8,0),(SE as u8,0),(SW as u8,0),(NW as u8,0)]); s }, jumps: vec![(-3,3),(3,3),(3,-3),(-3,-3)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "TF" => Some(Movement { slides: all_dirs_slide.clone(), jumps: vec![(-2,0),(0,2),(2,0),(0,-2),(-2,2),(2,2),(2,-2),(-2,-2),(-3,0),(0,3),(3,0),(0,-3)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "DG" => Some(Movement { slides: { let mut s: Vec<(u8,u8)> = vec![(N as u8,0),(E as u8,0),(S as u8,0),(W as u8,0),(NE as u8,0),(NW as u8,0)]; s.push((SE as u8,3)); s.push((SW as u8,3)); s }, jumps: vec![(-3,0),(0,3),(3,0),(0,-3),(-3,3),(-3,-3)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "LD" => Some(Movement { slides: all_dirs_slide.clone(), jumps: vec![(-3,0),(0,3),(3,0),(0,-3),(-3,3),(3,3),(3,-3),(-3,-3)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "GS" => Some(Movement { slides: { let mut s: Vec<(u8,u8)> = vec![(N as u8,0),(E as u8,0),(S as u8,0),(W as u8,0)]; s.push((SE as u8,2)); s.push((SW as u8,2)); s }, jumps: vec![(-2,2),(-2,-2)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "MF" => Some(Movement { slides: { let mut s: Vec<(u8,u8)> = vec![(N as u8,0),(E as u8,0),(S as u8,0),(W as u8,0),(NE as u8,0),(NW as u8,0)]; s.push((SE as u8,2)); s.push((SW as u8,2)); s }, jumps: vec![(-2,0)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "ML" => Some(Movement { slides: { let mut s: Vec<(u8,u8)> = vec![(N as u8,0),(E as u8,0),(S as u8,0),(W as u8,0),(NE as u8,0),(NW as u8,0),(SW as u8,0)]; s.push((SE as u8,2)); s }, jumps: vec![(-2,-2)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "MR" => Some(Movement { slides: { let mut s: Vec<(u8,u8)> = vec![(N as u8,0),(E as u8,0),(S as u8,0),(W as u8,0),(NE as u8,0),(NW as u8,0),(SE as u8,0)]; s.push((SW as u8,2)); s }, jumps: vec![(-2,2)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "GT" | "LL" => Some(Movement { slides: { let mut s = all_dirs_slide.clone(); s.push((E as u8,2)); s.push((W as u8,2)); s }, jumps: vec![(-2,0),(2,0)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "KM" => Some(Movement { slides: vec![(N as u8,0),(S as u8,0),(NE as u8,0),(NW as u8,0),(SE as u8,0),(SW as u8,0),(E as u8,3),(W as u8,3)], jumps: vec![(-3,0),(3,0)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "PM" => Some(Movement { slides: vec![(N as u8,0),(S as u8,0),(NE as u8,0),(NW as u8,0),(SE as u8,0),(SW as u8,0),(E as u8,3),(W as u8,3)], jumps: vec![(-3,3),(-3,-3)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "GM" => Some(Movement { slides: vec![(NE as u8,0),(NW as u8,0),(N as u8,0),(S as u8,0),(SE as u8,5),(SW as u8,5),(E as u8,5),(W as u8,5)], jumps: vec![(-3,3),(-3,-3),(-3,0)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "MT" => Some(Movement { slides: vec![(N as u8,0),(S as u8,0),(NE as u8,0),(NW as u8,0),(E as u8,3),(W as u8,3),(SE as u8,3),(SW as u8,3)], jumps: vec![(-2,0),(2,0),(-2,2),(-2,-2)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "PE" => Some(Movement { slides: vec![(N as u8,0),(S as u8,0),(NE as u8,0),(NW as u8,0),(SE as u8,5),(SW as u8,5),(E as u8,5),(W as u8,5)], jumps: vec![(-3,3),(-3,-3)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "GO" => Some(Movement { slides: vec![(SE as u8,3),(SW as u8,3),(N as u8,0),(S as u8,0)], jumps: vec![], hook: None, area: 0, range_capture: vec![], igui: false }),
        "+GEA" | "+GHK" => Some(Movement { slides: all_dirs_slide.clone(), jumps: if abbrev == "+GEA" { vec![(-2,2),(-2,-2)] } else { vec![(-2,0)] }, hook: None, area: 0, range_capture: vec![], igui: false }),
        "+HTK" => Some(Movement { slides: all_dirs_slide.clone(), jumps: vec![(-2,0),(0,2),(2,0),(0,-2),(-2,2),(2,2),(2,-2),(-2,-2)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "+KTE" | "+MCR" => Some(Movement { slides: all_dirs_slide.clone(), jumps: vec![(-3,0),(0,3),(3,0),(0,-3),(-3,3),(3,3),(3,-3),(-3,-3)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "+SPT" => Some(Movement { slides: all_dirs_slide.clone(), jumps: vec![(-3,0),(0,3),(3,0),(0,-3)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "+TRT" => Some(Movement { slides: all_dirs_slide.clone(), jumps: vec![(-2,0),(0,2),(2,0),(0,-2)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "+GDE" => Some(Movement { slides: all_dirs_slide.clone(), jumps: vec![(0,3),(0,-3)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "+GEL" => Some(Movement { slides: { let mut s: Vec<(u8,u8)> = vec![(NE as u8,3),(NW as u8,3)]; s.extend([(N as u8,0),(E as u8,0),(S as u8,0),(W as u8,0)]); s }, jumps: vec![], hook: None, area: 0, range_capture: vec![], igui: false }),
        "+ADR" => Some(Movement { slides: vec![(NE as u8,0),(SE as u8,0),(SW as u8,0),(NW as u8,0),(N as u8,0),(S as u8,0)], jumps: vec![], hook: None, area: 0, range_capture: vec![], igui: false }),
        "+HHO" => Some(Movement { slides: vec![(N as u8,0)], jumps: vec![(-2,1),(-2,-1),(2,1),(2,-1)], hook: None, area: 0, range_capture: vec![], igui: false }),
        "+RDM" => Some(Movement { slides: vec![(N as u8,3),(E as u8,2),(W as u8,2),(S as u8,0),(SE as u8,2),(SW as u8,2)], jumps: vec![], hook: None, area: 0, range_capture: vec![], igui: false }),
        "+FBI" => Some(Movement { slides: vec![(N as u8,0),(E as u8,0),(S as u8,0),(W as u8,0),(SE as u8,3),(SW as u8,3)], jumps: vec![], hook: None, area: 0, range_capture: vec![], igui: false }),
        _ => None,
    }
}

// ============================================================
// Initialize the piece store
// ============================================================
fn init_store() -> PieceStore {
    let count = PIECE_DEFS.len();
    let mut abbrev_to_id = HashMap::new();
    let mut movements = vec![Movement::empty()]; // index 0 unused
    let mut promo = vec![0u16]; // index 0 unused

    for (i, def) in PIECE_DEFS.iter().enumerate() {
        let id = (i + 1) as u16;
        abbrev_to_id.insert(def.abbrev.to_string(), id);
    }

    for (i, def) in PIECE_DEFS.iter().enumerate() {
        // Movement
        let mv = if let Some(manual) = manual_movement(def.abbrev) {
            manual
        } else if let Some(betza) = def.betza {
            parse_betza(betza).unwrap_or_else(Movement::empty)
        } else {
            Movement::empty()
        };
        movements.push(mv);

        // Promotion
        let promo_id = def.promotes_to
            .and_then(|ab| abbrev_to_id.get(ab))
            .copied()
            .unwrap_or(0);
        promo.push(promo_id);
    }

    PieceStore { count, abbrev_to_id, movements, promotes_to: promo }
}

// ============================================================
// Initial board setup
// ============================================================
pub const SETUP_RANKS: [&str; 12] = [
    "L  TS RR W  DM ML LO BC HR FR ED CD FT Q  RS LG G  K  CP G  RG RS Q  FT WO ED FR HR BC LO MR DM W  RR WT L",
    "RV WE TD FS CO RA FO MS RP RU SS GR RT BA BD WR S  NK DE S  GU YA BA RT GR SS RU RP MS FO RA CO FS TD FG RV",
    "GC SI RN RW BG RO LT LE BO WD FP RB OK PC WA FI C  KM PM C  FI WA PC OK RB FP WD BO RI TT RO BG RW RN SI GC",
    "SV VE N  PI CG PG H  O  CN SA SR GL LN CT GS VD WL GG VG WL VD GS CT LN GL SR SA CN O  H  PG CG PI N  VE SV",
    "CI CE B  R  WF FC MF VT SO LS CL CR RH HE VO GD GO DV DS GO GD VO HE RH CR CL LS SO VT MF FC WF R  B  CE CI",
    "WC WH DL SM PR WB FL EG FD PS FY ST BI WG F  KR CA GT LL HM PH F  WG BI ST FY PS FD EG FL WB PR SM DR WH WC",
    "TC VW SX DO FH VB AB EW WI CK OM CC WS ES VS NT TF PE MT TF NT VS SU NB CC OM CK WI EW AB VB FH DO SX VW TC",
    "EC BL EB HO OW CM CS SW BM BT OC SF BB OR SQ SN RD LI FE RD SN SQ OR BB SF OC BT BM SW CS CM OW HO EB VI EC",
    "CH SL VR WN RE M  SD HS GN OS EA BS SG LP T  BE I  GM GE I  BE T  LP SG BS EA OS GN HS SD M  RE WN VR SL CH",
    "LC MK VM OX LB VP VH BN DH DK SE HF EL SP VL TG SC LD DG SC TG VL SP EL HF SE DK DH BN VH VP LB OX VM MK RC",
    "P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P  P",
    ".  .  .  .  .  D  .  .  .  .  GB .  .  .  .  D  .  .  .  .  .  .  .  D  .  .  .  .  GB .  .  .  .  D  .  .",
];

pub fn parse_setup_rank(rank_str: &str) -> Vec<Option<u16>> {
    let s = store();
    rank_str.split_whitespace()
        .map(|tok| {
            if tok == "." { None }
            else { s.abbrev_to_id.get(tok).copied() }
        })
        .collect()
}
