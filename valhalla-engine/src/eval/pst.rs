//! Piece-Square Tables for the 14x14 four-player chess board.
//!
//! Base tables defined from Red's perspective (south, pushing north).
//! Other orientations derived via coordinate rotation:
//! - Red:    pst[rank][file]           (base)
//! - Blue:   pst[file][13-rank]        (90 CW)
//! - Yellow: pst[13-rank][13-file]     (180)
//! - Green:  pst[13-file][rank]        (270 CW)
//!
//! Pre-computed into `[Player][PieceType][square_index]` at init time.

use std::sync::OnceLock;

use crate::types::constants::BOARD_SIZE;
use crate::types::piece::PieceType;
use crate::types::player::{Player, ALL_PLAYERS};
use crate::types::square::Square;

/// Number of piece types with PST values (Pawn through PromotedQueen).
const NUM_PIECE_TYPES: usize = 7;
const TOTAL_SQUARES: usize = 196;

/// Pre-computed PST values: `PST[player][piece_type][square_index]`.
/// Access via `pst_value()`.
static PST_TABLES: OnceLock<[[[i16; TOTAL_SQUARES]; NUM_PIECE_TYPES]; 4]> = OnceLock::new();

/// Get the PST value for a piece on a square.
#[inline]
pub fn pst_value(player: Player, piece_type: PieceType, sq: Square) -> i16 {
    let tables = PST_TABLES.get_or_init(init_pst_tables);
    tables[player.index()][piece_type as usize][sq.index() as usize]
}

// Base PST tables from Red's perspective (14x14 grid, rank-major order).
// Row 0 = rank 0 (Red's back rank), Row 13 = rank 13 (Yellow's back rank).
// Invalid corner squares have value 0 (never accessed in practice).

// Pawns: center control, advancement bonus. Red pushes north (increasing rank).
#[rustfmt::skip]
const PAWN_BASE: [[i16; 14]; 14] = [
    // rank 0: back rank — pawns never here
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    // rank 1: Red pawn start rank — base position
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    // rank 2: one step forward
    [0, 0, 0, 2, 2, 4, 4, 4, 4, 2, 2, 0, 0, 0],
    // rank 3: two steps forward
    [0, 0, 0, 5, 5, 8, 10, 10, 8, 5, 5, 0, 0, 0],
    // rank 4: entering center zone
    [0, 0, 0, 8, 10, 15, 20, 20, 15, 10, 8, 0, 0, 0],
    // rank 5: mid-center
    [0, 0, 0, 10, 15, 22, 28, 28, 22, 15, 10, 0, 0, 0],
    // rank 6: deep center
    [0, 0, 0, 15, 20, 28, 35, 35, 28, 20, 15, 0, 0, 0],
    // rank 7: deep center
    [0, 0, 0, 18, 25, 32, 40, 40, 32, 25, 18, 0, 0, 0],
    // rank 8: promotion rank for Red (FFA) — big bonus
    [0, 0, 0, 50, 55, 60, 65, 65, 60, 55, 50, 0, 0, 0],
    // rank 9+: beyond promotion — shouldn't happen for Red pawns
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

// Knights: strong in center, weak on edges. Center of 14x14 is around ranks/files 5-8.
#[rustfmt::skip]
const KNIGHT_BASE: [[i16; 14]; 14] = [
    [0, 0, 0,-20,-15,-10,-10,-10,-10,-15,-20, 0, 0, 0],
    [0, 0, 0,-10, -5,  0,  0,  0,  0, -5,-10, 0, 0, 0],
    [0, 0, 0, -5,  0,  8, 10, 10,  8,  0, -5, 0, 0, 0],
    [0, 0, 0,  0,  5, 12, 18, 18, 12,  5,  0, 0, 0, 0],
    [0, 0, 0,  0,  8, 18, 25, 25, 18,  8,  0, 0, 0, 0],
    [0, 0, 0,  0, 10, 20, 28, 28, 20, 10,  0, 0, 0, 0],
    [0, 0, 0,  0, 10, 20, 30, 30, 20, 10,  0, 0, 0, 0],
    [0, 0, 0,  0, 10, 20, 30, 30, 20, 10,  0, 0, 0, 0],
    [0, 0, 0,  0, 10, 20, 28, 28, 20, 10,  0, 0, 0, 0],
    [0, 0, 0,  0,  8, 18, 25, 25, 18,  8,  0, 0, 0, 0],
    [0, 0, 0,  0,  5, 12, 18, 18, 12,  5,  0, 0, 0, 0],
    [0, 0, 0, -5,  0,  8, 10, 10,  8,  0, -5, 0, 0, 0],
    [0, 0, 0,-10, -5,  0,  0,  0,  0, -5,-10, 0, 0, 0],
    [0, 0, 0,-20,-15,-10,-10,-10,-10,-15,-20, 0, 0, 0],
];

// Bishops: slight center preference, diagonals.
#[rustfmt::skip]
const BISHOP_BASE: [[i16; 14]; 14] = [
    [0, 0, 0,-10, -5, -5, -5, -5, -5, -5,-10, 0, 0, 0],
    [0, 0, 0, -5,  5,  0,  0,  0,  0,  5, -5, 0, 0, 0],
    [0, 0, 0, -5,  0,  8,  5,  5,  8,  0, -5, 0, 0, 0],
    [0, 0, 0,  0,  5, 10, 12, 12, 10,  5,  0, 0, 0, 0],
    [0, 0, 0,  0,  8, 12, 15, 15, 12,  8,  0, 0, 0, 0],
    [0, 0, 0,  0, 10, 15, 18, 18, 15, 10,  0, 0, 0, 0],
    [0, 0, 0,  0, 10, 15, 20, 20, 15, 10,  0, 0, 0, 0],
    [0, 0, 0,  0, 10, 15, 20, 20, 15, 10,  0, 0, 0, 0],
    [0, 0, 0,  0, 10, 15, 18, 18, 15, 10,  0, 0, 0, 0],
    [0, 0, 0,  0,  8, 12, 15, 15, 12,  8,  0, 0, 0, 0],
    [0, 0, 0,  0,  5, 10, 12, 12, 10,  5,  0, 0, 0, 0],
    [0, 0, 0, -5,  0,  8,  5,  5,  8,  0, -5, 0, 0, 0],
    [0, 0, 0, -5,  5,  0,  0,  0,  0,  5, -5, 0, 0, 0],
    [0, 0, 0,-10, -5, -5, -5, -5, -5, -5,-10, 0, 0, 0],
];

// Rooks: 7th rank bonus for Red (rank 7-8 = deep penetration), open file valued.
#[rustfmt::skip]
const ROOK_BASE: [[i16; 14]; 14] = [
    [0, 0, 0,  0,  0,  0,  5,  5,  0,  0,  0, 0, 0, 0],
    [0, 0, 0,  0,  0,  0,  0,  0,  0,  0,  0, 0, 0, 0],
    [0, 0, 0, -2,  0,  0,  0,  0,  0,  0, -2, 0, 0, 0],
    [0, 0, 0, -2,  0,  0,  0,  0,  0,  0, -2, 0, 0, 0],
    [0, 0, 0, -2,  0,  0,  0,  0,  0,  0, -2, 0, 0, 0],
    [0, 0, 0, -2,  0,  0,  0,  0,  0,  0, -2, 0, 0, 0],
    [0, 0, 0,  0,  0,  5,  8,  8,  5,  0,  0, 0, 0, 0],
    [0, 0, 0,  5, 10, 15, 20, 20, 15, 10,  5, 0, 0, 0],
    [0, 0, 0,  5, 10, 15, 20, 20, 15, 10,  5, 0, 0, 0],
    [0, 0, 0,  0,  0,  5,  8,  8,  5,  0,  0, 0, 0, 0],
    [0, 0, 0, -2,  0,  0,  0,  0,  0,  0, -2, 0, 0, 0],
    [0, 0, 0, -2,  0,  0,  0,  0,  0,  0, -2, 0, 0, 0],
    [0, 0, 0,  0,  0,  0,  0,  0,  0,  0,  0, 0, 0, 0],
    [0, 0, 0,  0,  0,  0,  5,  5,  0,  0,  0, 0, 0, 0],
];

// Queen: minimal PST — avoid over-committing queen position. Slight center bonus.
#[rustfmt::skip]
const QUEEN_BASE: [[i16; 14]; 14] = [
    [0, 0, 0, -5, -5, -5, -5, -5, -5, -5, -5, 0, 0, 0],
    [0, 0, 0, -5,  0,  0,  0,  0,  0,  0, -5, 0, 0, 0],
    [0, 0, 0, -5,  0,  3,  3,  3,  3,  0, -5, 0, 0, 0],
    [0, 0, 0,  0,  0,  3,  5,  5,  3,  0,  0, 0, 0, 0],
    [0, 0, 0,  0,  0,  5,  8,  8,  5,  0,  0, 0, 0, 0],
    [0, 0, 0,  0,  0,  5,  8,  8,  5,  0,  0, 0, 0, 0],
    [0, 0, 0,  0,  0,  5, 10, 10,  5,  0,  0, 0, 0, 0],
    [0, 0, 0,  0,  0,  5, 10, 10,  5,  0,  0, 0, 0, 0],
    [0, 0, 0,  0,  0,  5,  8,  8,  5,  0,  0, 0, 0, 0],
    [0, 0, 0,  0,  0,  5,  8,  8,  5,  0,  0, 0, 0, 0],
    [0, 0, 0,  0,  0,  3,  5,  5,  3,  0,  0, 0, 0, 0],
    [0, 0, 0, -5,  0,  3,  3,  3,  3,  0, -5, 0, 0, 0],
    [0, 0, 0, -5,  0,  0,  0,  0,  0,  0, -5, 0, 0, 0],
    [0, 0, 0, -5, -5, -5, -5, -5, -5, -5, -5, 0, 0, 0],
];

// King: castled positions preferred (files 5-8, rank 0-1 for Red).
// Penalize center in opening — king should hide behind pawns.
#[rustfmt::skip]
const KING_BASE: [[i16; 14]; 14] = [
    [0, 0, 0, -5, 10, 20, 30, 30, 20, 10, -5, 0, 0, 0],
    [0, 0, 0,-10,  0, 10, 15, 15, 10,  0,-10, 0, 0, 0],
    [0, 0, 0,-20,-15,-10,-10,-10,-10,-15,-20, 0, 0, 0],
    [0, 0, 0,-25,-20,-15,-15,-15,-15,-20,-25, 0, 0, 0],
    [0, 0, 0,-30,-25,-20,-20,-20,-20,-25,-30, 0, 0, 0],
    [0, 0, 0,-30,-25,-20,-20,-20,-20,-25,-30, 0, 0, 0],
    [0, 0, 0,-30,-25,-20,-20,-20,-20,-25,-30, 0, 0, 0],
    [0, 0, 0,-30,-25,-20,-20,-20,-20,-25,-30, 0, 0, 0],
    [0, 0, 0,-30,-25,-20,-20,-20,-20,-25,-30, 0, 0, 0],
    [0, 0, 0,-30,-25,-20,-20,-20,-20,-25,-30, 0, 0, 0],
    [0, 0, 0,-25,-20,-15,-15,-15,-15,-20,-25, 0, 0, 0],
    [0, 0, 0,-20,-15,-10,-10,-10,-10,-15,-20, 0, 0, 0],
    [0, 0, 0,-10,  0, 10, 15, 15, 10,  0,-10, 0, 0, 0],
    [0, 0, 0, -5, 10, 20, 30, 30, 20, 10, -5, 0, 0, 0],
];

/// Look up base table value for Red at (rank, file).
fn base_value(piece_type: PieceType, rank: u8, file: u8) -> i16 {
    let r = rank as usize;
    let f = file as usize;
    match piece_type {
        PieceType::Pawn => PAWN_BASE[r][f],
        PieceType::Knight => KNIGHT_BASE[r][f],
        PieceType::Bishop => BISHOP_BASE[r][f],
        PieceType::Rook => ROOK_BASE[r][f],
        PieceType::Queen | PieceType::PromotedQueen => QUEEN_BASE[r][f],
        PieceType::King => KING_BASE[r][f],
    }
}

/// Rotate (rank, file) from Red's perspective to another player's.
///
/// Red (south, pushing north):  identity
/// Blue (west, pushing east):   90 CW  -> (file, 13-rank)
/// Yellow (north, pushing south): 180  -> (13-rank, 13-file)
/// Green (east, pushing west):  270 CW -> (13-file, rank)
fn rotated_coords(player: Player, rank: u8, file: u8) -> (u8, u8) {
    match player {
        Player::Red => (rank, file),
        Player::Blue => (file, 13 - rank),
        Player::Yellow => (13 - rank, 13 - file),
        Player::Green => (13 - file, rank),
    }
}

/// Initialize all PST tables by rotating base tables for each player.
fn init_pst_tables() -> [[[i16; TOTAL_SQUARES]; NUM_PIECE_TYPES]; 4] {
    let mut tables = [[[0i16; TOTAL_SQUARES]; NUM_PIECE_TYPES]; 4];

    let piece_types = [
        PieceType::Pawn,
        PieceType::Knight,
        PieceType::Bishop,
        PieceType::Rook,
        PieceType::Queen,
        PieceType::King,
        PieceType::PromotedQueen,
    ];

    for &player in &ALL_PLAYERS {
        for &pt in &piece_types {
            for rank in 0..BOARD_SIZE {
                for file in 0..BOARD_SIZE {
                    if !Square::is_valid(rank, file) {
                        continue; // Skip invalid corner squares
                    }
                    let sq_index = (rank as usize) * (BOARD_SIZE as usize) + (file as usize);
                    // Map this player's (rank, file) to Red's coordinate space
                    let (base_r, base_f) = rotated_coords(player, rank, file);
                    tables[player.index()][pt as usize][sq_index] = base_value(pt, base_r, base_f);
                }
            }
        }
    }

    tables
}

/// Sum PST values for all pieces of a given player.
pub fn pst_sum(state: &crate::game_state::GameState, player: Player) -> i32 {
    let mut total: i32 = 0;
    for (sq, piece_type) in state.board.pieces_for_player(player) {
        total += pst_value(player, piece_type, sq) as i32;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::GameState;
    use crate::types::square::Square;

    #[test]
    fn test_pst_tables_initialized() {
        // Force initialization and verify we can read values
        let val = pst_value(Player::Red, PieceType::Pawn, Square(1 * 14 + 5));
        // rank 1, file 5 — Red pawn start, should be 0
        assert_eq!(val, 0);
    }

    #[test]
    fn test_red_pawn_advancement_bonus() {
        // Pawn on rank 4 center should have higher PST than rank 1
        let start = pst_value(Player::Red, PieceType::Pawn, Square(1 * 14 + 6));
        let advanced = pst_value(Player::Red, PieceType::Pawn, Square(4 * 14 + 6));
        assert!(advanced > start, "Advanced pawn should have higher PST");
    }

    #[test]
    fn test_knight_center_bonus() {
        // Knight in center vs edge
        let center = pst_value(Player::Red, PieceType::Knight, Square(6 * 14 + 6));
        let edge = pst_value(Player::Red, PieceType::Knight, Square(0 * 14 + 3));
        assert!(
            center > edge,
            "Center knight should have higher PST than edge"
        );
    }

    #[test]
    fn test_rotational_symmetry_red_vs_yellow() {
        // Red's d2 (rank 1, file 3) pawn should equal Yellow's k13 (rank 12, file 10) pawn
        // Yellow is 180 rotation: (13-rank, 13-file) -> (12, 10) maps to base (1, 3)
        let red_val = pst_value(Player::Red, PieceType::Pawn, Square(1 * 14 + 3));
        let yellow_val = pst_value(Player::Yellow, PieceType::Pawn, Square(12 * 14 + 10));
        assert_eq!(
            red_val, yellow_val,
            "Red d2 should match Yellow k13 (180 rotation)"
        );
    }

    #[test]
    fn test_rotational_symmetry_red_vs_blue() {
        // Red's rank 1, file 5 maps to Blue's rotation
        // Blue (90 CW): Red's (r,f) is Blue's (f, 13-r) -> Blue at (5, 12)
        // So Red (1, 5) should match Blue (5, 12)
        let red_val = pst_value(Player::Red, PieceType::Knight, Square(1 * 14 + 5));
        let blue_val = pst_value(Player::Blue, PieceType::Knight, Square(5 * 14 + 12));
        assert_eq!(
            red_val, blue_val,
            "Red (1,5) should match Blue (5,12) via 90 CW rotation"
        );
    }

    #[test]
    fn test_rotational_symmetry_red_vs_green() {
        // Green (270 CW): Red's (r,f) is Green's (13-f, r)
        // Red (1, 5) -> Green (8, 1)
        let red_val = pst_value(Player::Red, PieceType::Knight, Square(1 * 14 + 5));
        let green_val = pst_value(Player::Green, PieceType::Knight, Square(8 * 14 + 1));
        assert_eq!(
            red_val, green_val,
            "Red (1,5) should match Green (8,1) via 270 CW rotation"
        );
    }

    #[test]
    fn test_starting_position_pst_symmetric() {
        let state = GameState::new();
        let red_pst = pst_sum(&state, Player::Red);
        let blue_pst = pst_sum(&state, Player::Blue);
        let yellow_pst = pst_sum(&state, Player::Yellow);
        let green_pst = pst_sum(&state, Player::Green);

        assert_eq!(
            red_pst, blue_pst,
            "Red and Blue PST sums should match at start"
        );
        assert_eq!(
            red_pst, yellow_pst,
            "Red and Yellow PST sums should match at start"
        );
        assert_eq!(
            red_pst, green_pst,
            "Red and Green PST sums should match at start"
        );
    }

    #[test]
    fn test_corner_squares_are_zero() {
        // Corner square (0, 0) should be 0
        let val = pst_value(Player::Red, PieceType::Knight, Square(0));
        assert_eq!(val, 0);
    }
}
