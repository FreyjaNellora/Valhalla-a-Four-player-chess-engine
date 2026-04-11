//! Swarm Layer 6: Chain Participation — the synthesis layer.
//!
//! For each contested square, recommends:
//! - Initiate: chain walk wins, low delta cost, healthy commitment
//! - Reinforce: marginal exchange, adding a piece would tip it
//! - Stay out: would lose, or costs too much elsewhere, or overextended

use super::ChainWalkResult;
use crate::types::Player;

/// Compute chain participation contribution and confidence.
///
/// Synthesizes layers 3-5: positive when good initiate/reinforce opportunities
/// exist, negative when all options are stay-out.
pub fn chain_participation(
    chain_results: &[ChainWalkResult],
    delta_contribution: f32,
    delta_confidence: f32,
    commitment_contribution: f32,
    commitment_confidence: f32,
    root_player: Player,
) -> (f32, f32) {
    if chain_results.is_empty() {
        return (0.0, 1.0);
    }

    let rp = root_player.index();
    let mut total_score = 0.0f32;
    let mut decisions = 0u32;

    for result in chain_results {
        if result.participants[rp] == 0 {
            // Not involved in this exchange
            continue;
        }

        decisions += 1;

        let wins_exchange = result.material_delta[rp] > 0;
        let even_exchange = result.material_delta[rp] == 0;
        let low_delta_cost = delta_contribution > -0.2;
        let healthy_commitment = commitment_contribution > -0.1;

        if wins_exchange && low_delta_cost && healthy_commitment {
            // Initiate: clear go signal
            total_score += 0.5;
        } else if (wins_exchange || even_exchange) && (low_delta_cost || healthy_commitment) {
            // Reinforce: marginal but worth considering
            total_score += 0.15;
        } else {
            // Stay out: too costly or losing
            total_score -= 0.3;
        }
    }

    if decisions == 0 {
        return (0.0, 0.7);
    }

    let contribution = total_score / decisions as f32;

    // Confidence: influenced by upstream layer confidence
    let upstream_conf = (delta_confidence + commitment_confidence) / 2.0;
    let confidence = (contribution.abs() * upstream_conf).clamp(0.2, 1.0);

    (contribution, confidence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swarm::ChainWalkResult;

    #[test]
    fn test_no_chains() {
        let (c, _) = chain_participation(&[], 0.0, 1.0, 0.0, 1.0, Player::Red);
        assert_eq!(c, 0.0);
    }

    #[test]
    fn test_winning_exchange_initiates() {
        let result = ChainWalkResult {
            material_delta: [300, -300, 0, 0], // Red wins
            participants: [1, 1, 0, 0],
            resolved: true,
        };
        let (c, _) = chain_participation(
            &[result],
            0.0, // low delta cost
            0.7,
            0.1, // healthy commitment
            0.7,
            Player::Red,
        );
        assert!(c > 0.0, "Winning exchange should recommend initiate");
    }

    #[test]
    fn test_losing_exchange_stays_out() {
        let result = ChainWalkResult {
            material_delta: [-300, 300, 0, 0], // Red loses
            participants: [1, 1, 0, 0],
            resolved: true,
        };
        let (c, _) = chain_participation(
            &[result],
            -0.5, // high delta cost
            0.7,
            -0.3, // overextended
            0.7,
            Player::Red,
        );
        assert!(c < 0.0, "Losing exchange should recommend stay out");
    }
}
