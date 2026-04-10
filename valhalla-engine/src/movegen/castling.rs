use super::attack::is_square_attacked_by;
use crate::game_state::GameState;
/// Castling move generation for all 8 variants (2 per player).
/// All square indices from 4PC_RULES_REFERENCE.md Section 6.3.
use crate::types::{Move, MoveBuffer, Player, Square, ALL_PLAYERS};

/// Castling definition: all the squares involved in one castling move.
struct CastleDef {
    player: Player,
    kingside: bool,
    king_from: u8,
    king_to: u8,
    rook_from: u8,
    rook_to: u8,
    /// Squares that must be empty (between king and rook, excluding endpoints).
    empty: &'static [u8],
    /// Squares the king passes through (including from and to), checked for attacks.
    pass_through: &'static [u8],
    /// Bit index in castling_rights.
    rights_bit: u8,
}

/// All 8 castling variants.
static CASTLE_DEFS: [CastleDef; 8] = [
    // Red Kingside: h1(7)->j1(9), k1(10)->i1(8)
    CastleDef {
        player: Player::Red,
        kingside: true,
        king_from: 7,
        king_to: 9,
        rook_from: 10,
        rook_to: 8,
        empty: &[8, 9],
        pass_through: &[7, 8, 9],
        rights_bit: 0,
    },
    // Red Queenside: h1(7)->f1(5), d1(3)->g1(6)
    CastleDef {
        player: Player::Red,
        kingside: false,
        king_from: 7,
        king_to: 5,
        rook_from: 3,
        rook_to: 6,
        empty: &[4, 5, 6],
        pass_through: &[7, 6, 5],
        rights_bit: 1,
    },
    // Blue Kingside: a7(84)->a5(56), a4(42)->a6(70)
    CastleDef {
        player: Player::Blue,
        kingside: true,
        king_from: 84,
        king_to: 56,
        rook_from: 42,
        rook_to: 70,
        empty: &[56, 70],
        pass_through: &[84, 70, 56],
        rights_bit: 2,
    },
    // Blue Queenside: a7(84)->a9(112), a11(140)->a8(98)
    CastleDef {
        player: Player::Blue,
        kingside: false,
        king_from: 84,
        king_to: 112,
        rook_from: 140,
        rook_to: 98,
        empty: &[98, 112, 126],
        pass_through: &[84, 98, 112],
        rights_bit: 3,
    },
    // Yellow Kingside: g14(188)->e14(186), d14(185)->f14(187)
    CastleDef {
        player: Player::Yellow,
        kingside: true,
        king_from: 188,
        king_to: 186,
        rook_from: 185,
        rook_to: 187,
        empty: &[186, 187],
        pass_through: &[188, 187, 186],
        rights_bit: 4,
    },
    // Yellow Queenside: g14(188)->i14(190), k14(192)->h14(189)
    CastleDef {
        player: Player::Yellow,
        kingside: false,
        king_from: 188,
        king_to: 190,
        rook_from: 192,
        rook_to: 189,
        empty: &[189, 190, 191],
        pass_through: &[188, 189, 190],
        rights_bit: 5,
    },
    // Green Kingside: n8(111)->n10(139), n11(153)->n9(125)
    CastleDef {
        player: Player::Green,
        kingside: true,
        king_from: 111,
        king_to: 139,
        rook_from: 153,
        rook_to: 125,
        empty: &[125, 139],
        pass_through: &[111, 125, 139],
        rights_bit: 6,
    },
    // Green Queenside: n8(111)->n6(83), n4(55)->n7(97)
    CastleDef {
        player: Player::Green,
        kingside: false,
        king_from: 111,
        king_to: 83,
        rook_from: 55,
        rook_to: 97,
        empty: &[69, 83, 97],
        pass_through: &[111, 97, 83],
        rights_bit: 7,
    },
];

/// Generate castling moves for the given player.
pub fn generate_castling_moves(state: &GameState, player: Player, buf: &mut impl MoveBuffer) {
    for def in &CASTLE_DEFS {
        if def.player != player {
            continue;
        }

        // Check castling right
        if state.castling_rights & (1 << def.rights_bit) == 0 {
            continue;
        }

        // Check all "must be empty" squares
        let mut path_clear = true;
        for &sq_idx in def.empty {
            if state.board.get(Square(sq_idx)).is_some() {
                path_clear = false;
                break;
            }
        }
        if !path_clear {
            continue;
        }

        // Check king does not pass through or land on attacked squares.
        // Must check all THREE opponents.
        let mut safe = true;
        for &sq_idx in def.pass_through {
            let sq = Square(sq_idx);
            for opponent in ALL_PLAYERS {
                if opponent == player || !state.is_active(opponent) {
                    continue;
                }
                if is_square_attacked_by(state, sq, opponent) {
                    safe = false;
                    break;
                }
            }
            if !safe {
                break;
            }
        }
        if !safe {
            continue;
        }

        // Generate the castling move
        let from = Square(def.king_from);
        let to = Square(def.king_to);
        if def.kingside {
            buf.push_move(Move::castle_kingside(from, to));
        } else {
            buf.push_move(Move::castle_queenside(from, to));
        }
    }
}

/// Get the rook squares for a castling move (rook_from, rook_to).
/// Used by make_move to also move the rook.
pub fn get_castle_rook_squares(player: Player, kingside: bool) -> (Square, Square) {
    for def in &CASTLE_DEFS {
        if def.player == player && def.kingside == kingside {
            return (Square(def.rook_from), Square(def.rook_to));
        }
    }
    unreachable!("Invalid castling request")
}

/// All rook starting squares for castling rights revocation.
/// (square_index, player_index, is_kingside)
pub const ROOK_START_SQUARES: [(u8, usize, bool); 8] = [
    (10, 0, true),   // Red KS rook at k1
    (3, 0, false),   // Red QS rook at d1
    (42, 1, true),   // Blue KS rook at a4
    (140, 1, false), // Blue QS rook at a11
    (185, 2, true),  // Yellow KS rook at d14
    (192, 2, false), // Yellow QS rook at k14
    (153, 3, true),  // Green KS rook at n11
    (55, 3, false),  // Green QS rook at n4
];

/// Get the castling rights bit for a given player and side.
pub fn castling_bit(player: Player, kingside: bool) -> u8 {
    let base = player.index() * 2;
    if kingside {
        base as u8
    } else {
        base as u8 + 1
    }
}
