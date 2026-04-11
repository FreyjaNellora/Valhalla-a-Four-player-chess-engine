//! Swarm Layer 1: Force Ratio.
//!
//! For each contested square (2+ players have influence), compute weighted
//! force balance. Contribution is positive if root player has favorable
//! balance, negative otherwise. Confidence reflects how decisive the balance is.

use super::ContestInfo;
use crate::types::Player;

/// Compute force ratio contribution and confidence.
///
/// Scans all contested squares. For each, compares root player's weighted
/// force (influence × piece values) against opponents' combined force.
pub fn force_ratio(contests: &[ContestInfo], root_player: Player) -> (f32, f32) {
    if contests.is_empty() {
        return (0.0, 1.0); // No contested squares = quiet, high confidence
    }

    let rp = root_player.index();
    let mut total_contribution = 0.0f32;
    let mut total_weight = 0.0f32;

    for info in contests {
        // Root player's force: attacker count weighted by piece values
        let my_force = info.attacker_values[rp] as f32;
        let my_count = info.attacker_counts[rp] as f32;

        // Opponents' combined force
        let mut opp_force = 0.0f32;
        let mut opp_count = 0.0f32;
        for pi in 0..4 {
            if pi != rp {
                opp_force += info.attacker_values[pi] as f32;
                opp_count += info.attacker_counts[pi] as f32;
            }
        }

        // Skip squares where root player has no involvement
        if my_count < 0.01 && opp_count < 0.01 {
            continue;
        }

        // Defender value makes the square more important
        let square_importance = 1.0 + info.defender_value as f32 / 900.0;

        // Force ratio: positive = we have more, negative = they have more
        let total_force = my_force + opp_force;
        if total_force < 1.0 {
            continue;
        }

        let ratio = (my_force - opp_force) / total_force;
        total_contribution += ratio * square_importance;
        total_weight += square_importance;
    }

    if total_weight < 0.01 {
        return (0.0, 1.0);
    }

    let contribution = total_contribution / total_weight;

    // Confidence: high when the balance is decisively one-sided
    let confidence = contribution.abs().min(1.0);

    (contribution, confidence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swarm::ContestInfo;
    use crate::types::Square;

    #[test]
    fn test_empty_contests() {
        let (c, conf) = force_ratio(&[], Player::Red);
        assert_eq!(c, 0.0);
        assert_eq!(conf, 1.0);
    }

    #[test]
    fn test_red_dominant_force() {
        let contest = ContestInfo {
            square: Square(70),
            attacker_values: [900, 100, 0, 0], // Red has much more
            attacker_counts: [2, 1, 0, 0],
            defender_value: 100,
            defender_player: Some(Player::Blue),
            influence: [2.0, 0.5, 0.0, 0.0],
        };
        let (c, conf) = force_ratio(&[contest], Player::Red);
        assert!(c > 0.0, "Red should have positive force ratio");
        assert!(conf > 0.5, "Should be confident with lopsided force");
    }

    #[test]
    fn test_red_weak_force() {
        let contest = ContestInfo {
            square: Square(70),
            attacker_values: [100, 500, 300, 0],
            attacker_counts: [1, 2, 1, 0],
            defender_value: 100,
            defender_player: Some(Player::Red),
            influence: [0.5, 2.0, 1.0, 0.0],
        };
        let (c, _) = force_ratio(&[contest], Player::Red);
        assert!(
            c < 0.0,
            "Red should have negative force ratio when outnumbered"
        );
    }

    #[test]
    fn test_balanced_force() {
        let contest = ContestInfo {
            square: Square(70),
            attacker_values: [500, 500, 0, 0],
            attacker_counts: [1, 1, 0, 0],
            defender_value: 0,
            defender_player: None,
            influence: [1.0, 1.0, 0.0, 0.0],
        };
        let (c, conf) = force_ratio(&[contest], Player::Red);
        assert!(c.abs() < 0.01, "Balanced force should give near-zero ratio");
        assert!(conf < 0.1, "Low confidence when balanced");
    }
}
