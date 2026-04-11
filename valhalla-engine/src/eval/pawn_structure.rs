//! Pawn structure evaluation component.
//!
//! Doubled, isolated, passed pawns, and pawn chains.
//! All direction-aware for four-player orientations.

use crate::game_state::GameState;
use crate::types::piece::PieceType;
use crate::types::player::{Player, ALL_PLAYERS};
use crate::types::square::Square;
use crate::types::Score;

/// Doubled pawn penalty per extra pawn on the same lane.
const DOUBLED_PENALTY: Score = -15;

/// Isolated pawn penalty (no friendly pawn on adjacent lanes).
const ISOLATED_PENALTY: Score = -20;

/// Passed pawn base bonus.
const PASSED_BONUS_BASE: Score = 30;

/// Passed pawn advancement bonus per rank advanced.
const PASSED_BONUS_PER_RANK: Score = 5;

/// Connected pawn (chain) bonus per link.
const CHAIN_BONUS: Score = 10;

/// Compute pawn structure score for `player`.
pub fn pawn_structure(state: &GameState, player: Player) -> Score {
    let (dr, df) = player.push_direction();
    let is_rank_pusher = dr != 0; // Red/Yellow push along rank

    // Collect pawn positions
    let mut pawns: arrayvec::ArrayVec<Square, 16> = arrayvec::ArrayVec::new();
    for (sq, pt) in state.board.pieces_for_player(player) {
        if pt == PieceType::Pawn {
            pawns.push(sq);
        }
    }

    let mut score: Score = 0;

    score += doubled_score(&pawns, is_rank_pusher);
    score += isolated_score(&pawns, is_rank_pusher);
    score += passed_score(state, player, &pawns, is_rank_pusher, dr, df);
    score += chain_score(state, player, &pawns);

    score
}

/// Doubled pawns: multiple pawns on the same "file" (lane perpendicular to push).
fn doubled_score(pawns: &[Square], is_rank_pusher: bool) -> Score {
    let mut lane_counts = [0u8; 14];
    for &sq in pawns {
        let lane = if is_rank_pusher { sq.file() } else { sq.rank() };
        lane_counts[lane as usize] += 1;
    }

    let mut score: Score = 0;
    for &count in &lane_counts {
        if count > 1 {
            score += DOUBLED_PENALTY * (count - 1) as Score;
        }
    }
    score
}

/// Isolated pawns: a pawn whose adjacent lanes have no friendly pawn.
fn isolated_score(pawns: &[Square], is_rank_pusher: bool) -> Score {
    let mut lane_occupied = [false; 14];
    for &sq in pawns {
        let lane = if is_rank_pusher { sq.file() } else { sq.rank() };
        lane_occupied[lane as usize] = true;
    }

    let mut score: Score = 0;
    for &sq in pawns {
        let lane = if is_rank_pusher { sq.file() } else { sq.rank() } as usize;
        let has_left = lane > 0 && lane_occupied[lane - 1];
        let has_right = lane < 13 && lane_occupied[lane + 1];
        if !has_left && !has_right {
            score += ISOLATED_PENALTY;
        }
    }
    score
}

/// Passed pawns: no enemy pawns ahead on the same or adjacent lanes.
fn passed_score(
    state: &GameState,
    player: Player,
    pawns: &[Square],
    is_rank_pusher: bool,
    dr: i8,
    df: i8,
) -> Score {
    // Collect all enemy pawn positions
    let mut enemy_pawns: arrayvec::ArrayVec<Square, 48> = arrayvec::ArrayVec::new();
    for &opp in &ALL_PLAYERS {
        if opp != player && state.is_active(opp) {
            for (sq, pt) in state.board.pieces_for_player(opp) {
                if pt == PieceType::Pawn {
                    enemy_pawns.push(sq);
                }
            }
        }
    }

    let mut score: Score = 0;

    for &pawn_sq in pawns {
        let pawn_lane = if is_rank_pusher {
            pawn_sq.file()
        } else {
            pawn_sq.rank()
        };
        let pawn_advance = if is_rank_pusher {
            pawn_sq.rank()
        } else {
            pawn_sq.file()
        };

        let is_passed = !enemy_pawns.iter().any(|&esq| {
            let enemy_lane = if is_rank_pusher {
                esq.file()
            } else {
                esq.rank()
            };
            let enemy_advance = if is_rank_pusher {
                esq.rank()
            } else {
                esq.file()
            };

            // Check if enemy pawn is on same or adjacent lane
            let lane_diff = (enemy_lane as i8 - pawn_lane as i8).abs();
            if lane_diff > 1 {
                return false;
            }

            // Check if enemy pawn is "ahead" of our pawn
            if dr > 0 || df > 0 {
                enemy_advance > pawn_advance
            } else {
                enemy_advance < pawn_advance
            }
        });

        if is_passed {
            // Compute advancement distance from start
            let start = player.pawn_start_rank_or_file();
            let advancement = if dr > 0 || df > 0 {
                pawn_advance.saturating_sub(start) as Score
            } else {
                start.saturating_sub(pawn_advance) as Score
            };
            score += PASSED_BONUS_BASE + PASSED_BONUS_PER_RANK * advancement;
        }
    }

    score
}

/// Pawn chains: pawns protecting each other diagonally along push direction.
fn chain_score(state: &GameState, player: Player, pawns: &[Square]) -> Score {
    let capture_dirs = player.capture_directions();
    let mut score: Score = 0;

    for &pawn_sq in pawns {
        // Check if any pawn is protecting this one from behind
        // A pawn at sq is protected if there's a friendly pawn at sq - capture_dir
        for &(cr, cf) in &capture_dirs {
            if let Some(behind_sq) = pawn_sq.offset(-cr, -cf) {
                if let Some(piece) = state.board.get(behind_sq) {
                    if piece.player == player && piece.piece_type == PieceType::Pawn {
                        score += CHAIN_BONUS;
                        break; // Count each pawn's chain membership once
                    }
                }
            }
        }
    }

    score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::GameState;

    #[test]
    fn test_starting_position_pawn_structure_symmetric() {
        let state = GameState::new();
        let red = pawn_structure(&state, Player::Red);
        let blue = pawn_structure(&state, Player::Blue);
        let yellow = pawn_structure(&state, Player::Yellow);
        let green = pawn_structure(&state, Player::Green);

        assert_eq!(
            red, blue,
            "Red and Blue pawn structure should match at start"
        );
        assert_eq!(red, yellow);
        assert_eq!(red, green);
    }

    #[test]
    fn test_no_doubled_at_start() {
        // Starting position has one pawn per file — no doubled pawns
        let state = GameState::new();
        let pawns: arrayvec::ArrayVec<Square, 16> = state
            .board
            .pieces_for_player(Player::Red)
            .filter(|(_, pt)| *pt == PieceType::Pawn)
            .map(|(sq, _)| sq)
            .collect();
        assert_eq!(doubled_score(&pawns, true), 0);
    }

    #[test]
    fn test_no_isolated_at_start() {
        // Starting position has continuous pawn line — no isolated pawns
        let state = GameState::new();
        let pawns: arrayvec::ArrayVec<Square, 16> = state
            .board
            .pieces_for_player(Player::Red)
            .filter(|(_, pt)| *pt == PieceType::Pawn)
            .map(|(sq, _)| sq)
            .collect();
        assert_eq!(isolated_score(&pawns, true), 0);
    }
}
