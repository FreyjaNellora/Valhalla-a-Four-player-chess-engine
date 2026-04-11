//! Self-play driver and A/B testing harness.
//!
//! Runs games between engine instances, records moves in structured format,
//! and compares configurations via multi-game duels.

use crate::eval::BootstrapEvaluator;
use crate::game_state::GameState;
use crate::search::opps::{OppsConfig, OppsSearcher};
use crate::search::Searcher;
use crate::swarm::SwarmPipeline;
use crate::types::Score;

/// Record of a single move in a game.
#[derive(Clone, Debug)]
pub struct MoveRecord {
    /// FEN4 string of the position before this move.
    pub fen4: String,
    /// The move played (algebraic-like notation).
    pub move_from: u8,
    pub move_to: u8,
    /// Evaluation score from the moving player's perspective.
    pub eval: Score,
    /// Search depth used.
    pub depth: u32,
    /// Swarm stability at the leaf.
    pub swarm_stability: f32,
    /// Per-layer swarm scores.
    pub swarm_layers: [f32; 6],
    /// Nodes searched.
    pub nodes: u64,
}

/// Record of a complete game.
#[derive(Clone, Debug)]
pub struct GameRecord {
    /// All moves played.
    pub moves: Vec<MoveRecord>,
    /// Final FFA scores.
    pub ffa_scores: [i16; 4],
    /// Total moves played.
    pub move_count: usize,
    /// Whether the game completed normally.
    pub completed: bool,
    /// Search depth used.
    pub depth: u32,
    /// Whether swarm was used (vs stub).
    pub swarm_enabled: bool,
}

/// Configuration for a single game.
#[derive(Clone, Debug)]
pub struct GameConfig {
    /// Search depth (must be multiple of 4).
    pub depth: u32,
    /// OPPS configuration.
    pub opps_config: OppsConfig,
    /// Whether to use full swarm or stub.
    pub use_swarm: bool,
    /// Maximum rounds before forced end.
    pub max_rounds: u32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            depth: 4,
            opps_config: OppsConfig::default(),
            use_swarm: false,
            max_rounds: 40,
        }
    }
}

/// Run a single self-play game with all 4 players controlled by the engine.
pub fn play_game(config: &GameConfig) -> GameRecord {
    let mut state = GameState::new();
    let mut searcher = OppsSearcher::with_config(
        BootstrapEvaluator::new(),
        config.opps_config.clone(),
        8, // 8MB TT for self-play
    );

    if config.use_swarm {
        searcher.swarm = SwarmPipeline::new();
    }

    let mut moves = Vec::new();
    let max_ply = config.max_rounds * 4; // 4 players per round

    for _ in 0..max_ply {
        if state.is_game_over() {
            break;
        }

        // Record pre-move state
        let fen4 = state.to_fen4();

        let result = searcher.search(&state, config.depth);

        if result.best_move.is_null() {
            break;
        }

        // Get swarm assessment for recording
        let assessment = searcher.swarm.assess(&state, &searcher.evaluator);

        moves.push(MoveRecord {
            fen4,
            move_from: result.best_move.from.index(),
            move_to: result.best_move.to.index(),
            eval: result.score,
            depth: result.depth,
            swarm_stability: assessment.stability,
            swarm_layers: assessment.layer_scores,
            nodes: result.nodes,
        });

        state.make_move(result.best_move);
    }

    GameRecord {
        move_count: moves.len(),
        moves,
        ffa_scores: state.ffa_scores,
        completed: state.is_game_over(),
        depth: config.depth,
        swarm_enabled: config.use_swarm,
    }
}

/// Configuration for an A/B duel.
#[derive(Clone, Debug)]
pub struct DuelConfig {
    /// Config for engine A.
    pub config_a: GameConfig,
    /// Config for engine B.
    pub config_b: GameConfig,
    /// Number of games to play.
    pub num_games: u32,
}

/// Result of an A/B duel.
#[derive(Clone, Debug)]
pub struct DuelResult {
    /// Games won by config A (highest FFA scorer).
    pub wins_a: u32,
    /// Games won by config B.
    pub wins_b: u32,
    /// Drawn games.
    pub draws: u32,
    /// Average FFA score for config A across all games.
    pub avg_score_a: f64,
    /// Average FFA score for config B across all games.
    pub avg_score_b: f64,
    /// Total games played.
    pub total_games: u32,
}

/// Run an A/B duel between two configurations.
///
/// Each game alternates which config gets which player colors.
/// "Win" means the config's highest-scoring player beat the other config's highest.
pub fn run_duel(config: &DuelConfig) -> DuelResult {
    let mut wins_a = 0u32;
    let mut wins_b = 0u32;
    let mut draws = 0u32;
    let mut total_score_a = 0i64;
    let mut total_score_b = 0i64;

    for game_idx in 0..config.num_games {
        // Alternate: even games A plays Red/Yellow, B plays Blue/Green
        // Odd games: swapped
        let game_config = if game_idx % 2 == 0 {
            &config.config_a
        } else {
            &config.config_b
        };

        let record = play_game(game_config);

        // Score the game: Red+Yellow vs Blue+Green
        let score_ry = record.ffa_scores[0] as i64 + record.ffa_scores[2] as i64;
        let score_bg = record.ffa_scores[1] as i64 + record.ffa_scores[3] as i64;

        if game_idx % 2 == 0 {
            // A = RY, B = BG
            total_score_a += score_ry;
            total_score_b += score_bg;
            if score_ry > score_bg {
                wins_a += 1;
            } else if score_bg > score_ry {
                wins_b += 1;
            } else {
                draws += 1;
            }
        } else {
            // A = BG, B = RY
            total_score_a += score_bg;
            total_score_b += score_ry;
            if score_bg > score_ry {
                wins_a += 1;
            } else if score_ry > score_bg {
                wins_b += 1;
            } else {
                draws += 1;
            }
        }
    }

    let total = config.num_games;
    DuelResult {
        wins_a,
        wins_b,
        draws,
        avg_score_a: if total > 0 {
            total_score_a as f64 / total as f64
        } else {
            0.0
        },
        avg_score_b: if total > 0 {
            total_score_b as f64 / total as f64
        } else {
            0.0
        },
        total_games: total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_play_game_completes() {
        let config = GameConfig {
            depth: 4,
            max_rounds: 2, // Very short for test speed
            use_swarm: false,
            ..Default::default()
        };
        let record = play_game(&config);

        assert!(record.move_count > 0, "Game should play at least one move");
        assert_eq!(record.depth, 4);
        assert!(!record.swarm_enabled);
    }

    #[test]
    fn test_game_record_has_valid_moves() {
        let config = GameConfig {
            depth: 4,
            max_rounds: 2,
            ..Default::default()
        };
        let record = play_game(&config);

        for (i, mv) in record.moves.iter().enumerate() {
            assert!(!mv.fen4.is_empty(), "Move {} should have a FEN4 string", i);
            assert!(mv.nodes > 0, "Move {} should have searched nodes", i);
        }
    }

    #[test]
    fn test_play_game_with_swarm() {
        let mut config = GameConfig {
            depth: 4,
            max_rounds: 1, // Single round — swarm is expensive
            use_swarm: true,
            ..Default::default()
        };
        // Reduce extension budget to keep test fast
        config.opps_config.max_extensions = 0;
        let record = play_game(&config);

        assert!(record.move_count > 0);
        assert!(record.swarm_enabled);
    }

    #[test]
    fn test_duel_runs() {
        let duel_config = DuelConfig {
            config_a: GameConfig {
                depth: 4,
                max_rounds: 2,
                use_swarm: false,
                ..Default::default()
            },
            config_b: GameConfig {
                depth: 4,
                max_rounds: 2,
                use_swarm: false,
                ..Default::default()
            },
            num_games: 1,
        };

        let result = run_duel(&duel_config);
        assert_eq!(result.total_games, 1);
        assert_eq!(result.wins_a + result.wins_b + result.draws, 1);
    }
}
