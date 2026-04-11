//! Material balance evaluation component.
//!
//! Sums centipawn values for side_to_move, subtracts average of active opponents.
//! DKW players excluded from opponent average.

use crate::game_state::GameState;
use crate::types::{Player, Score, ALL_PLAYERS};

/// Compute material balance from the perspective of `state.side_to_move()`.
///
/// Your material minus the average of active opponents' material.
/// DKW players are excluded (not competitive opponents).
pub fn material_balance(state: &GameState) -> Score {
    let stm = state.side_to_move();
    let my_material = sum_material(state, stm);

    let mut opp_total: Score = 0;
    let mut opp_count: i32 = 0;
    for &p in &ALL_PLAYERS {
        if p != stm && state.is_active(p) {
            opp_total += sum_material(state, p);
            opp_count += 1;
        }
    }

    if opp_count == 0 {
        return my_material;
    }

    my_material - opp_total / opp_count as Score
}

/// Sum centipawn values of all pieces belonging to `player`.
fn sum_material(state: &GameState, player: Player) -> Score {
    let mut total: Score = 0;
    for (_sq, piece_type) in state.board.pieces_for_player(player) {
        total += piece_type.eval_centipawns() as Score;
    }
    total
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::GameState;

    #[test]
    fn test_starting_position_material_zero() {
        let state = GameState::new();
        // Starting position is symmetric — material balance should be 0
        assert_eq!(material_balance(&state), 0);
    }

    #[test]
    fn test_material_after_losing_pawn() {
        let mut state = GameState::new();
        // Remove a Red pawn — Red's material drops, should be negative for Red
        // Find a Red pawn square
        let mut pawn_sq = None;
        for (sq, pt) in state.board.pieces_for_player(Player::Red) {
            if pt == crate::types::PieceType::Pawn {
                pawn_sq = Some(sq);
                break;
            }
        }
        let sq = pawn_sq.expect("Red should have pawns");
        state.board.remove_piece(sq);

        // Red to move — should see negative material balance
        assert!(material_balance(&state) < 0);
    }
}
