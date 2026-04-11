//! Swarm Layer 5: Commitment Counting.
//!
//! Across all contested squares: what fraction of total piece value is
//! tied up in exchanges? High ratio = overextended. Low ratio = flexible.

use super::ChainWalkResult;
use crate::game_state::GameState;
use crate::types::Player;

/// Compute commitment contribution and confidence.
///
/// Negative contribution when the root player is overextended
/// (commitment ratio > 0.4 or engaged on > 2 fronts).
pub fn commitment_count(
    state: &GameState,
    chain_results: &[ChainWalkResult],
    root_player: Player,
) -> (f32, f32) {
    if chain_results.is_empty() {
        return (0.0, 1.0);
    }

    let rp = root_player.index();

    // Total piece value for root player
    let mut total_value = 0i32;
    for (_, piece_type) in state.board.pieces_for_player(root_player) {
        total_value += piece_type.eval_centipawns() as i32;
    }

    if total_value == 0 {
        return (0.0, 1.0);
    }

    // Committed value: sum of piece values in exchanges
    let mut committed_value = 0i32;
    let mut front_count = 0u32;

    for result in chain_results {
        if result.participants[rp] > 0 {
            front_count += 1;
            // Approximate committed value from material delta magnitude
            committed_value += result.material_delta[rp].unsigned_abs() as i32;
        }
    }

    let commitment_ratio = committed_value as f32 / total_value as f32;

    // Overextended: high ratio or too many fronts
    let contribution = if commitment_ratio > 0.4 || front_count > 2 {
        // Penalty scales with overextension
        -(commitment_ratio * 0.5 + (front_count as f32 - 2.0).max(0.0) * 0.1)
    } else if front_count == 0 {
        0.0 // Not engaged anywhere
    } else {
        // Healthy commitment
        0.1 * (1.0 - commitment_ratio)
    };

    let confidence = if front_count > 0 { 0.7 } else { 0.5 };

    (contribution, confidence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_state::GameState;
    use crate::swarm::ChainWalkResult;

    #[test]
    fn test_no_chains() {
        let state = GameState::new();
        let (c, _) = commitment_count(&state, &[], Player::Red);
        assert_eq!(c, 0.0);
    }

    #[test]
    fn test_healthy_commitment() {
        let state = GameState::new();
        let result = ChainWalkResult {
            material_delta: [100, -100, 0, 0],
            participants: [1, 1, 0, 0],
            resolved: true,
        };
        let (c, _) = commitment_count(&state, &[result], Player::Red);
        // Should be slightly positive (healthy single-front engagement)
        assert!(c >= 0.0, "Single front engagement should be healthy");
    }
}
