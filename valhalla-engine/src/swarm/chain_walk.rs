//! Swarm Layer 3: Chain Walk — branchless capture resolution.
//!
//! SUPER-SOMA adapted for 4 players. Resolves capture sequences statically:
//! collect all attackers per player (sorted by value ascending), walk turn
//! order R→B→Y→G, each player captures if gain >= risk.
//!
//! O(n) per contested square. Most positions have 2-4 contested squares.

use arrayvec::ArrayVec;

use super::{ChainWalkResult, ContestInfo};
use crate::game_state::GameState;
use crate::types::piece::PieceType;
use crate::types::player::{Player, ALL_PLAYERS};

/// Maximum attackers per player on a single square.
const MAX_ATTACKERS: usize = 8;

/// Run chain walk on all contested squares.
pub fn chain_walk_all(
    state: &GameState,
    contests: &[ContestInfo],
) -> ArrayVec<ChainWalkResult, 64> {
    let mut results = ArrayVec::new();
    for info in contests {
        results.push(chain_walk(state, info));
    }
    results
}

/// Run chain walk on a single contested square.
///
/// Collects all attackers per player, sorted by piece value ascending.
/// Walks through turn order: each player decides whether to capture.
pub fn chain_walk(state: &GameState, info: &ContestInfo) -> ChainWalkResult {
    // Collect attackers per player, sorted by piece value (cheapest first)
    let mut attackers: [ArrayVec<i16, MAX_ATTACKERS>; 4] = [
        ArrayVec::new(),
        ArrayVec::new(),
        ArrayVec::new(),
        ArrayVec::new(),
    ];

    for &player in &ALL_PLAYERS {
        if !state.is_active(player) {
            continue;
        }
        let pi = player.index();
        if info.attacker_counts[pi] == 0 {
            continue;
        }

        // Gather actual attacker values from piece list
        let mut values = ArrayVec::<i16, MAX_ATTACKERS>::new();
        for (piece_sq, piece_type) in state.board.pieces_for_player(player) {
            if piece_type == PieceType::King {
                continue; // Kings don't participate in exchanges
            }
            if super::can_attack(state, piece_sq, piece_type, info.square, player) {
                values.push(piece_type.eval_centipawns());
            }
        }
        values.sort_unstable(); // cheapest first
        attackers[pi] = values;
    }

    let mut material_delta = [0i16; 4];
    let mut participants = [0u8; 4];

    // Current piece on the square
    let mut on_square: Option<(usize, i16)> = info
        .defender_player
        .map(|p| (p.index(), info.defender_value));

    // Walk rounds until no one captures
    let max_rounds = 8; // Safety limit
    for _ in 0..max_rounds {
        let mut any_captured = false;

        for &player in &ALL_PLAYERS {
            let pi = player.index();
            if attackers[pi].is_empty() {
                continue;
            }

            if let Some((defender_pi, defender_val)) = on_square {
                if defender_pi == pi {
                    continue; // Can't capture own piece
                }

                let attacker_val = attackers[pi][0];

                // Decision: capture if gain >= risk, or if we have more attackers backing us up
                let gain = defender_val;
                let risk = attacker_val;
                let backup = attackers[pi].len() > 1;

                if gain >= risk || (gain > 0 && backup) {
                    // Capture!
                    material_delta[pi] += gain;
                    material_delta[defender_pi] -= defender_val;
                    on_square = Some((pi, attacker_val));
                    attackers[pi].remove(0);
                    participants[pi] += 1;
                    any_captured = true;
                }
            }
        }

        if !any_captured {
            break;
        }
    }

    ChainWalkResult {
        material_delta,
        participants,
        resolved: true,
    }
}

/// Score the chain walk results from root player's perspective.
pub fn chain_walk_score(results: &[ChainWalkResult], root_player: Player) -> (f32, f32) {
    if results.is_empty() {
        return (0.0, 1.0);
    }

    let rp = root_player.index();
    let mut total_delta = 0.0f32;
    let mut active_chains = 0u32;

    for result in results {
        let any_activity: u8 = result.participants.iter().sum();
        if any_activity == 0 {
            continue;
        }
        active_chains += 1;

        // Normalize by 100cp (pawn value) for scaling
        total_delta += result.material_delta[rp] as f32 / 100.0;
    }

    if active_chains == 0 {
        return (0.0, 1.0);
    }

    let contribution = total_delta / active_chains as f32;
    // Confidence is high when exchanges are clearly resolved
    let confidence = if contribution.abs() > 0.5 { 0.9 } else { 0.5 };

    (contribution, confidence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::swarm::ContestInfo;
    use crate::types::Square;

    fn make_contest(
        defender: Option<Player>,
        defender_val: i16,
        atk_vals: [i16; 4],
        atk_counts: [u8; 4],
    ) -> ContestInfo {
        ContestInfo {
            square: Square(70),
            attacker_values: atk_vals,
            attacker_counts: atk_counts,
            defender_value: defender_val,
            defender_player: defender,
            influence: [1.0; 4],
        }
    }

    #[test]
    fn test_no_attackers_no_exchange() {
        let info = make_contest(Some(Player::Blue), 500, [0, 0, 0, 0], [0, 0, 0, 0]);
        let state = GameState::new();
        let result = chain_walk(&state, &info);
        assert_eq!(result.material_delta, [0, 0, 0, 0]);
    }

    #[test]
    fn test_chain_walk_score_empty() {
        let (c, conf) = chain_walk_score(&[], Player::Red);
        assert_eq!(c, 0.0);
        assert_eq!(conf, 1.0);
    }
}
