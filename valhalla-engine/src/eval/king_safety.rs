//! Structural king safety evaluation component.
//!
//! Pawn shield integrity, open files near king, castling status.
//! NO tactical threats — swarm's domain (ADR-008).

use crate::game_state::GameState;
use crate::types::piece::PieceType;
use crate::types::player::Player;
use crate::types::square::Square;
use crate::types::Score;

/// Pawn shield: present = +15cp, missing = -30cp.
const SHIELD_PRESENT: Score = 15;
const SHIELD_MISSING: Score = -30;

/// Open file near king penalty.
const OPEN_FILE_PENALTY: Score = -25;

/// Castling bonuses/penalties.
const CASTLING_RIGHTS_RETAINED: Score = 10;
const CASTLED_BONUS: Score = 30;
const RIGHTS_LOST_UNCASTLED: Score = -20;

/// Compute structural king safety for `player` from their own perspective.
///
/// Returns a centipawn score: positive = safe, negative = exposed.
pub fn king_safety(state: &GameState, player: Player) -> Score {
    let king_sq = match state.board.king_square(player) {
        Some(sq) => sq,
        None => return 0, // Eliminated — no king to protect
    };

    let mut score: Score = 0;

    score += pawn_shield_score(state, player, king_sq);
    score += open_files_score(state, player, king_sq);
    score += castling_score(state, player, king_sq);

    score
}

/// Check the 3 squares directly in front of the king for friendly pawns.
/// "In front" depends on player orientation.
fn pawn_shield_score(state: &GameState, player: Player, king_sq: Square) -> Score {
    let (dr, df) = player.push_direction();
    let mut score: Score = 0;

    // Three shield squares: directly ahead, and one step to each side
    let shield_offsets: [(i8, i8); 3] = if dr != 0 {
        // Red/Yellow: push along rank, shield is perpendicular (files)
        [(dr, -1), (dr, 0), (dr, 1)]
    } else {
        // Blue/Green: push along file, shield is perpendicular (ranks)
        [(-1, df), (0, df), (1, df)]
    };

    for &(r_off, f_off) in &shield_offsets {
        if let Some(shield_sq) = king_sq.offset(r_off, f_off) {
            if let Some(piece) = state.board.get(shield_sq) {
                if piece.player == player && piece.piece_type == PieceType::Pawn {
                    score += SHIELD_PRESENT;
                    continue;
                }
            }
            score += SHIELD_MISSING;
        }
        // Off the board = no penalty (king is on the edge, one less square to worry about)
    }

    score
}

/// Check if the files/ranks near the king lack friendly pawns (open files).
/// Scans the 3 columns (for Red/Yellow) or rows (for Blue/Green) around the king.
fn open_files_score(state: &GameState, player: Player, king_sq: Square) -> Score {
    let (dr, _df) = player.push_direction();
    let mut score: Score = 0;

    if dr != 0 {
        // Red/Yellow: "files" are actual files. Check files king_file-1, king_file, king_file+1
        let king_file = king_sq.file();
        for f_offset in -1i8..=1 {
            let file = king_file as i8 + f_offset;
            if !(0..14).contains(&file) {
                continue;
            }
            if !has_friendly_pawn_on_file(state, player, file as u8) {
                score += OPEN_FILE_PENALTY;
            }
        }
    } else {
        // Blue/Green: "files" are ranks for them. Check ranks king_rank-1, king_rank, king_rank+1
        let king_rank = king_sq.rank();
        for r_offset in -1i8..=1 {
            let rank = king_rank as i8 + r_offset;
            if !(0..14).contains(&rank) {
                continue;
            }
            if !has_friendly_pawn_on_rank(state, player, rank as u8) {
                score += OPEN_FILE_PENALTY;
            }
        }
    }

    score
}

/// Check if player has any pawn on the given file.
fn has_friendly_pawn_on_file(state: &GameState, player: Player, file: u8) -> bool {
    for (sq, pt) in state.board.pieces_for_player(player) {
        if pt == PieceType::Pawn && sq.file() == file {
            return true;
        }
    }
    false
}

/// Check if player has any pawn on the given rank.
fn has_friendly_pawn_on_rank(state: &GameState, player: Player, rank: u8) -> bool {
    for (sq, pt) in state.board.pieces_for_player(player) {
        if pt == PieceType::Pawn && sq.rank() == rank {
            return true;
        }
    }
    false
}

/// Castling status evaluation.
///
/// Detects whether the player has castled by checking if the king is on
/// a known castled square (moved from starting position to a castled location).
fn castling_score(state: &GameState, player: Player, king_sq: Square) -> Score {
    let (king_start, castled_squares) = king_positions(player);

    // Check if king has castled (on a known castled square)
    for &csq in &castled_squares {
        if king_sq == csq {
            return CASTLED_BONUS;
        }
    }

    // Check castling rights
    let player_rights_mask = 0x03u8 << (player.index() * 2);
    let has_rights = (state.castling_rights & player_rights_mask) != 0;

    if has_rights {
        CASTLING_RIGHTS_RETAINED
    } else if king_sq == king_start {
        // Lost rights but king still on starting square — bad
        RIGHTS_LOST_UNCASTLED
    } else {
        // King moved somewhere else (not castled, not starting) — neutral
        0
    }
}

/// Returns (king_start_square, [castled_kingside_sq, castled_queenside_sq]) for a player.
fn king_positions(player: Player) -> (Square, [Square; 2]) {
    match player {
        Player::Red => (
            Square::from_rank_file(0, 7).unwrap(), // e1 equivalent
            [
                Square::from_rank_file(0, 9).unwrap(), // g1 (kingside)
                Square::from_rank_file(0, 5).unwrap(), // c1 (queenside)
            ],
        ),
        Player::Blue => (
            Square::from_rank_file(7, 0).unwrap(), // starting king
            [
                Square::from_rank_file(9, 0).unwrap(), // kingside
                Square::from_rank_file(5, 0).unwrap(), // queenside
            ],
        ),
        Player::Yellow => (
            Square::from_rank_file(13, 6).unwrap(), // starting king
            [
                Square::from_rank_file(13, 4).unwrap(), // kingside
                Square::from_rank_file(13, 8).unwrap(), // queenside
            ],
        ),
        Player::Green => (
            Square::from_rank_file(6, 13).unwrap(), // starting king
            [
                Square::from_rank_file(4, 13).unwrap(), // kingside
                Square::from_rank_file(8, 13).unwrap(), // queenside
            ],
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::GameState;

    #[test]
    fn test_starting_position_king_safety_symmetric() {
        let state = GameState::new();
        let red = king_safety(&state, Player::Red);
        let blue = king_safety(&state, Player::Blue);
        let yellow = king_safety(&state, Player::Yellow);
        let green = king_safety(&state, Player::Green);

        assert_eq!(red, blue, "Red and Blue king safety should match at start");
        assert_eq!(
            red, yellow,
            "Red and Yellow king safety should match at start"
        );
        assert_eq!(
            red, green,
            "Red and Green king safety should match at start"
        );
    }

    #[test]
    fn test_king_safety_positive_at_start() {
        let state = GameState::new();
        // At start, king has full pawn shield and castling rights — should be positive
        let score = king_safety(&state, Player::Red);
        assert!(
            score > 0,
            "King safety at start should be positive, got {}",
            score
        );
    }
}
