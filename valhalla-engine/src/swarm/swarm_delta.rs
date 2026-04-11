//! Swarm Layer 4: Swarm-Delta.
//!
//! For each piece identified as a chain walk participant: does removing it
//! from its current position flip zone ownership? Check influence advantage
//! change for neighboring squares.

use super::{ChainWalkResult, ContestInfo};
use crate::game_state::GameState;
use crate::influence::InfluenceMap;
use crate::types::piece::KING_OFFSETS;
use crate::types::Player;

/// Compute swarm-delta contribution and confidence.
///
/// Checks whether chain walk participants are defending important squares
/// elsewhere. Negative contribution means engaging costs us defensive coverage.
pub fn swarm_delta(
    state: &GameState,
    chain_results: &[ChainWalkResult],
    contests: &[ContestInfo],
    influence: &InfluenceMap,
    root_player: Player,
) -> (f32, f32) {
    if chain_results.is_empty() {
        return (0.0, 1.0);
    }

    let rp = root_player.index();
    let mut total_delta = 0.0f32;
    let mut checked_count = 0u32;

    for (i, result) in chain_results.iter().enumerate() {
        if result.participants[rp] == 0 {
            continue; // Root player not involved in this chain
        }

        let contest = &contests[i];

        // Check neighboring squares of the contested square
        // If our influence advantage decreases significantly, that's a cost
        for &(dr, df) in &KING_OFFSETS {
            if let Some(neighbor) = contest.square.offset(dr, df) {
                let our_influence = influence.get(neighbor, root_player);
                let advantage = influence.advantage(neighbor, root_player);

                // If we have positive advantage on a neighbor and we're committing
                // pieces to the contested square, we might lose that advantage
                if advantage > 0.5 && our_influence > 0.3 {
                    // Check if there's something valuable to defend there
                    if let Some(piece) = state.board.get(neighbor) {
                        if piece.player == root_player {
                            // We have a piece here that depends on our influence
                            let exposure_cost = piece.piece_type.eval_centipawns() as f32 / 900.0;
                            total_delta -= exposure_cost * 0.2;
                        }
                    }
                }
            }
        }
        checked_count += 1;
    }

    if checked_count == 0 {
        return (0.0, 1.0);
    }

    let contribution = total_delta / checked_count as f32;
    let confidence = contribution.abs().clamp(0.3, 1.0);

    (contribution, confidence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::GameState;
    use crate::influence::InfluenceMap;

    #[test]
    fn test_no_chains_no_delta() {
        let state = GameState::new();
        let influence = InfluenceMap::compute(&state);
        let (c, conf) = swarm_delta(&state, &[], &[], &influence, Player::Red);
        assert_eq!(c, 0.0);
        assert_eq!(conf, 1.0);
    }
}
