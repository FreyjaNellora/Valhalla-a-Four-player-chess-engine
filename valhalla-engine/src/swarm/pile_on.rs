//! Swarm Layer 2: Pile-On Detection.
//!
//! For contested squares with 3+ players' attackers, identifies the lurker
//! (cheapest attackers) and suckers (most committed). Uses Lanchester
//! square law: effective force scales with influence².

use super::ContestInfo;
use crate::types::Player;

/// Compute pile-on contribution and confidence.
pub fn pile_on(contests: &[ContestInfo], root_player: Player) -> (f32, f32) {
    if contests.is_empty() {
        return (0.0, 1.0);
    }

    let rp = root_player.index();
    let mut total_contribution = 0.0f32;
    let mut pile_on_count = 0u32;

    for info in contests {
        // Count players with attackers
        let active_attackers: u8 = info.attacker_counts.iter().filter(|&&c| c > 0).count() as u8;

        if active_attackers < 3 {
            continue; // Not a pile-on, just a duel
        }

        pile_on_count += 1;

        // Find lurker (least committed) and suckers (most committed)
        // Commitment = attacker_values (total piece value tied up)
        let mut sorted_players: Vec<(usize, i16)> = (0..4)
            .filter(|&pi| info.attacker_counts[pi] > 0)
            .map(|pi| (pi, info.attacker_values[pi]))
            .collect();
        sorted_players.sort_by_key(|&(_, v)| v);

        if sorted_players.is_empty() {
            continue;
        }

        let lurker_idx = sorted_players[0].0;

        // Lanchester: effective force = influence²
        let my_effective = info.influence[rp] * info.influence[rp];
        let total_effective: f32 = (0..4)
            .map(|pi| info.influence[pi] * info.influence[pi])
            .sum();

        if total_effective < 0.01 {
            continue;
        }

        // Positive if root player is the lurker, negative if they're a sucker
        let contribution = if lurker_idx == rp {
            // We're the lurker — good position
            0.5 * (1.0 - my_effective / total_effective)
        } else if info.attacker_counts[rp] > 0 {
            // We're in the pile-on but not the lurker — potentially a sucker
            let our_commitment = info.attacker_values[rp] as f32;
            let lurker_commitment = sorted_players[0].1 as f32;
            if lurker_commitment > 0.01 {
                -0.3 * (our_commitment / lurker_commitment).min(2.0)
            } else {
                -0.3
            }
        } else {
            0.0 // We're not involved
        };

        total_contribution += contribution;
    }

    if pile_on_count == 0 {
        return (0.0, 0.5); // No pile-ons, moderate confidence
    }

    let avg = total_contribution / pile_on_count as f32;
    let confidence = avg.abs().min(1.0);
    (avg, confidence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swarm::ContestInfo;
    use crate::types::Square;

    #[test]
    fn test_no_pile_on_in_duel() {
        let contest = ContestInfo {
            square: Square(70),
            attacker_values: [300, 500, 0, 0], // only 2 players
            attacker_counts: [1, 1, 0, 0],
            defender_value: 100,
            defender_player: Some(Player::Red),
            influence: [1.0, 1.5, 0.0, 0.0],
        };
        let (c, _) = pile_on(&[contest], Player::Red);
        assert!(c.abs() < 0.01, "Duels should not produce pile-on signal");
    }

    #[test]
    fn test_lurker_gets_positive() {
        let contest = ContestInfo {
            square: Square(70),
            attacker_values: [100, 500, 300, 0], // Red is cheapest
            attacker_counts: [1, 2, 1, 0],
            defender_value: 100,
            defender_player: None,
            influence: [0.5, 2.0, 1.0, 0.0],
        };
        let (c, _) = pile_on(&[contest], Player::Red);
        assert!(
            c > 0.0,
            "Lurker (cheapest attacker) should get positive contribution"
        );
    }
}
