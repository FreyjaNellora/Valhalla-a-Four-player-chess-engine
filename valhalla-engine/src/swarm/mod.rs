//! Swarm module — six-layer tactical resolution pipeline.
//!
//! Replaces quiescence search. Assesses whether a leaf position is
//! tactically resolved and provides a stability signal for search extensions.
//!
//! Layers:
//! 1. Force Ratio — material balance at contested squares
//! 2. Pile-On — multiple attackers converging, Lanchester square law
//! 3. Chain Walk — branchless capture resolution (SUPER-SOMA for 4 players)
//! 4. Swarm-Delta — removing chain participants flips zone ownership?
//! 5. Commitment — fraction of total piece value tied up in exchanges
//! 6. Chain Participation — synthesis: initiate/reinforce/stay-out

pub mod aggregation;
pub mod chain_participation;
pub mod chain_walk;
pub mod commitment;
pub mod force_ratio;
pub mod pile_on;
pub mod swarm_delta;

use arrayvec::ArrayVec;

use crate::eval::Evaluator;
use crate::game_state::GameState;
use crate::influence::InfluenceMap;
use crate::movegen::is_square_attacked_by;
use crate::types::player::{Player, ALL_PLAYERS};
use crate::types::square::{Square, IS_VALID};
use crate::types::Score;

/// Maximum contested squares to track per position.
const MAX_CONTESTS: usize = 64;

/// Result of swarm tactical assessment at a leaf node.
#[derive(Clone, Copy, Debug)]
pub struct SwarmAssessment {
    /// Final score incorporating base eval + tactical adjustments.
    pub score: Score,
    /// How tactically resolved this position is (0.0 = chaotic, 1.0 = quiet).
    pub stability: f32,
    /// Per-layer contribution scores (for diagnostics/tuning).
    pub layer_scores: [f32; 6],
    /// Per-layer confidence values (for diagnostics/tuning).
    pub layer_confidences: [f32; 6],
}

/// Information about a contested square (2+ players have influence).
#[derive(Clone, Copy, Debug)]
pub struct ContestInfo {
    /// The contested square.
    pub square: Square,
    /// Total attacker value per player (sum of eval_centipawns of attacking pieces).
    pub attacker_values: [i16; 4],
    /// Number of attacking pieces per player.
    pub attacker_counts: [u8; 4],
    /// Value of the piece occupying this square (0 if empty).
    pub defender_value: i16,
    /// Player owning the piece on this square (None if empty).
    pub defender_player: Option<Player>,
    /// Influence values from the influence map.
    pub influence: [f32; 4],
}

/// Result of a chain walk on one contested square.
#[derive(Clone, Copy, Debug)]
pub struct ChainWalkResult {
    /// Net material change per player from the exchange sequence.
    pub material_delta: [i16; 4],
    /// How many pieces each player committed to the exchange.
    pub participants: [u8; 4],
    /// Whether the chain terminated cleanly.
    pub resolved: bool,
}

/// Configuration for the swarm pipeline.
#[derive(Clone, Debug)]
pub struct SwarmConfig {
    /// Per-layer weights for aggregation.
    pub weights: [f32; 6],
    /// Minimum weight for a layer to affect stability calculation.
    pub confidence_threshold: f32,
    /// If true, skip layers 3-6 when layers 1-2 produce high confidence.
    pub progressive_eval: bool,
    /// Whether this is a stub (pass-through) pipeline.
    pub is_stub: bool,
}

impl Default for SwarmConfig {
    fn default() -> Self {
        Self {
            weights: [1.0, 1.0, 1.5, 0.8, 0.6, 0.5],
            confidence_threshold: 0.1,
            progressive_eval: false,
            is_stub: false,
        }
    }
}

/// The swarm tactical resolution pipeline.
pub struct SwarmPipeline {
    /// Configuration.
    pub config: SwarmConfig,
}

impl SwarmPipeline {
    /// Create a stub (pass-through) pipeline.
    pub fn new_stub() -> Self {
        Self {
            config: SwarmConfig {
                is_stub: true,
                ..Default::default()
            },
        }
    }

    /// Create a full swarm pipeline with default configuration.
    pub fn new() -> Self {
        Self {
            config: SwarmConfig::default(),
        }
    }

    /// Assess a leaf position's tactical state.
    pub fn assess(&self, state: &GameState, evaluator: &dyn Evaluator) -> SwarmAssessment {
        let base_score = evaluator.evaluate(state);

        if self.config.is_stub {
            return SwarmAssessment {
                score: base_score,
                stability: 1.0,
                layer_scores: [0.0; 6],
                layer_confidences: [1.0; 6],
            };
        }

        let root_player = state.side_to_move();
        let influence = InfluenceMap::compute(state);
        let contests = find_contested_squares(state, &influence);

        if contests.is_empty() {
            return SwarmAssessment {
                score: base_score,
                stability: 1.0,
                layer_scores: [0.0; 6],
                layer_confidences: [1.0; 6],
            };
        }

        // Layer 1: Force Ratio
        let (c1, conf1) = force_ratio::force_ratio(&contests, root_player);

        // Layer 2: Pile-On Detection
        let (c2, conf2) = pile_on::pile_on(&contests, root_player);

        // Progressive eval: if L1+L2 are highly confident, skip L3-L6
        if self.config.progressive_eval && conf1 > 0.9 && conf2 > 0.9 {
            let contributions = [c1, c2, 0.0, 0.0, 0.0, 0.0];
            let confidences = [conf1, conf2, 1.0, 1.0, 1.0, 1.0];
            let (composite, stability) = aggregation::aggregate(
                &contributions,
                &confidences,
                &self.config.weights,
                self.config.confidence_threshold,
            );
            return SwarmAssessment {
                score: base_score + (composite * 100.0) as Score,
                stability,
                layer_scores: contributions,
                layer_confidences: confidences,
            };
        }

        // Layer 3: Chain Walk
        let chain_results = chain_walk::chain_walk_all(state, &contests);
        let (c3, conf3) = chain_walk::chain_walk_score(&chain_results, root_player);

        // Layer 4: Swarm-Delta
        let (c4, conf4) =
            swarm_delta::swarm_delta(state, &chain_results, &contests, &influence, root_player);

        // Layer 5: Commitment Counting
        let (c5, conf5) = commitment::commitment_count(state, &chain_results, root_player);

        // Layer 6: Chain Participation
        let (c6, conf6) = chain_participation::chain_participation(
            &chain_results,
            c4,
            conf4,
            c5,
            conf5,
            root_player,
        );

        let contributions = [c1, c2, c3, c4, c5, c6];
        let confidences = [conf1, conf2, conf3, conf4, conf5, conf6];

        let (composite, stability) = aggregation::aggregate(
            &contributions,
            &confidences,
            &self.config.weights,
            self.config.confidence_threshold,
        );

        SwarmAssessment {
            score: base_score + (composite * 100.0) as Score,
            stability,
            layer_scores: contributions,
            layer_confidences: confidences,
        }
    }
}

impl Default for SwarmPipeline {
    fn default() -> Self {
        Self::new()
    }
}

/// Find squares where 2+ players have significant influence.
pub fn find_contested_squares(
    state: &GameState,
    influence: &InfluenceMap,
) -> ArrayVec<ContestInfo, MAX_CONTESTS> {
    let mut contests = ArrayVec::new();
    let influence_threshold = 0.3f32;

    for (sq_idx, &valid) in IS_VALID.iter().enumerate() {
        if !valid {
            continue;
        }

        // Count how many players have significant influence
        let mut players_with_influence = 0u8;
        for pi in 0..4 {
            if influence.grid[sq_idx][pi] >= influence_threshold {
                players_with_influence += 1;
            }
        }

        if players_with_influence < 2 {
            continue;
        }

        let sq = Square::from_index_unchecked(sq_idx as u8);

        // Compute attacker details
        let mut attacker_values = [0i16; 4];
        let mut attacker_counts = [0u8; 4];

        for &player in &ALL_PLAYERS {
            if !state.is_active(player) {
                continue;
            }
            let pi = player.index();
            if is_square_attacked_by(state, sq, player) {
                // Count actual attackers via piece scan
                for (piece_sq, piece_type) in state.board.pieces_for_player(player) {
                    if can_attack(state, piece_sq, piece_type, sq, player) {
                        attacker_values[pi] += piece_type.eval_centipawns();
                        attacker_counts[pi] += 1;
                    }
                }
            }
        }

        // Get defender info
        let (defender_value, defender_player) = if let Some(piece) = state.board.get(sq) {
            (piece.piece_type.eval_centipawns(), Some(piece.player))
        } else {
            (0, None)
        };

        let mut inf = [0.0f32; 4];
        inf.copy_from_slice(&influence.grid[sq_idx]);

        contests.push(ContestInfo {
            square: sq,
            attacker_values,
            attacker_counts,
            defender_value,
            defender_player,
            influence: inf,
        });

        if contests.is_full() {
            break;
        }
    }

    contests
}

/// Check if a specific piece can attack a target square.
fn can_attack(
    state: &GameState,
    piece_sq: Square,
    piece_type: crate::types::PieceType,
    target: Square,
    player: Player,
) -> bool {
    use crate::types::piece::{KING_OFFSETS, KNIGHT_OFFSETS};
    use crate::types::PieceType;

    let dr = target.rank() as i8 - piece_sq.rank() as i8;
    let df = target.file() as i8 - piece_sq.file() as i8;

    match piece_type {
        PieceType::Pawn => {
            let caps = player.capture_directions();
            caps.iter().any(|&(cr, cf)| cr == dr && cf == df)
        }
        PieceType::Knight => KNIGHT_OFFSETS.iter().any(|&(kr, kf)| kr == dr && kf == df),
        PieceType::King => KING_OFFSETS.iter().any(|&(kr, kf)| kr == dr && kf == df),
        _ if piece_type.is_slider() => {
            // Check if target is on a valid ray and unblocked
            for &(ray_dr, ray_df) in piece_type.slide_directions() {
                if is_along_ray(dr, df, ray_dr, ray_df) {
                    // Walk the ray to check for blockers
                    let mut cur = piece_sq;
                    loop {
                        cur = match cur.offset(ray_dr, ray_df) {
                            Some(s) => s,
                            None => break,
                        };
                        if cur == target {
                            return true;
                        }
                        if state.board.get(cur).is_some() {
                            break; // Blocked
                        }
                    }
                }
            }
            false
        }
        _ => false,
    }
}

/// Check if a delta (dr, df) lies along a ray direction.
fn is_along_ray(dr: i8, df: i8, ray_dr: i8, ray_df: i8) -> bool {
    if dr == 0 && df == 0 {
        return false;
    }
    // Must be in the same direction and same ratio
    if ray_dr == 0 {
        return dr == 0 && df.signum() == ray_df.signum();
    }
    if ray_df == 0 {
        return df == 0 && dr.signum() == ray_dr.signum();
    }
    // Diagonal: |dr| == |df| and same sign pattern
    dr.abs() == df.abs() && dr.signum() == ray_dr.signum() && df.signum() == ray_df.signum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::BootstrapEvaluator;
    use crate::game_state::GameState;

    #[test]
    fn test_stub_returns_eval_score() {
        let state = GameState::new();
        let eval = BootstrapEvaluator::new();
        let swarm = SwarmPipeline::new_stub();
        let assessment = swarm.assess(&state, &eval);
        let expected = eval.evaluate(&state);
        assert_eq!(assessment.score, expected);
    }

    #[test]
    fn test_stub_stability_always_one() {
        let state = GameState::new();
        let eval = BootstrapEvaluator::new();
        let swarm = SwarmPipeline::new_stub();
        let assessment = swarm.assess(&state, &eval);
        assert_eq!(assessment.stability, 1.0);
    }

    #[test]
    fn test_stub_is_stub() {
        let swarm = SwarmPipeline::new_stub();
        assert!(swarm.config.is_stub);
    }

    #[test]
    fn test_full_is_not_stub() {
        let swarm = SwarmPipeline::new();
        assert!(!swarm.config.is_stub);
    }

    #[test]
    fn test_find_contested_starting_position() {
        let state = GameState::new();
        let influence = InfluenceMap::compute(&state);
        let contests = find_contested_squares(&state, &influence);
        // Starting position has contested center squares
        // Don't assert exact count, just that some exist
        assert!(
            !contests.is_empty(),
            "Starting position should have contested squares"
        );
    }

    #[test]
    fn test_full_pipeline_starting_position() {
        let state = GameState::new();
        let eval = BootstrapEvaluator::new();
        let swarm = SwarmPipeline::new();
        let assessment = swarm.assess(&state, &eval);
        // Should produce a finite score and valid stability
        assert!(assessment.score.abs() < 10000);
        assert!((0.0..=1.0).contains(&assessment.stability));
    }
}
