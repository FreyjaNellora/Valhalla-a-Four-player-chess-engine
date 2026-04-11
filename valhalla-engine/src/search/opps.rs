//! OPPS — Opponent-Pruning Paranoid Search.
//!
//! Paranoid negamax with alpha-beta pruning. Root player maximizes,
//! all opponents minimize. Parameters n1/l1/l2 control opponent move pruning.
//!
//! Co-developed with Swarm: every leaf is assessed by the swarm pipeline.

use arrayvec::ArrayVec;

use crate::eval::Evaluator;
use crate::game_state::GameState;
use crate::movegen::generate_legal_moves;
use crate::swarm::SwarmPipeline;
use crate::tt::{TTFlag, TranspositionTable};
use crate::types::{Move, Player, Score, MAX_MOVES, SCORE_DRAW, SCORE_MATE, SCORE_NEG_INFINITY};

use super::history::HistoryTable;
use super::killer::KillerTable;
use super::move_order::score_moves;
use super::{SearchResult, Searcher};

/// OPPS search configuration.
#[derive(Clone, Debug)]
pub struct OppsConfig {
    /// Node budget (soft limit). Default: 50,000.
    pub n1: u32,
    /// Max opponent moves at first round of opponent play. Default: 12.
    pub l1: usize,
    /// Max opponent moves at deeper levels. Default: 4.
    pub l2: usize,
    /// Swarm stability threshold for extensions (Phase 3c). Default: 0.5.
    pub stability_threshold: f32,
    /// Maximum extension rounds (each = 4 plies). Default: 2.
    pub max_extensions: u32,
}

impl Default for OppsConfig {
    fn default() -> Self {
        Self {
            n1: 50_000,
            l1: 12,
            l2: 4,
            stability_threshold: 0.5,
            max_extensions: 2,
        }
    }
}

/// OPPS searcher implementing the frozen Searcher trait.
pub struct OppsSearcher<E: Evaluator> {
    /// Search parameters.
    pub config: OppsConfig,
    /// Positional evaluator (bootstrap or NNUE).
    pub evaluator: E,
    /// Transposition table.
    pub tt: TranspositionTable,
    /// Killer move table.
    pub killers: KillerTable,
    /// History heuristic table.
    pub history: HistoryTable,
    /// Swarm tactical resolution pipeline.
    pub swarm: SwarmPipeline,
    /// Nodes visited in current search.
    nodes: u64,
    /// The player we are maximizing for (set at search entry).
    root_player: Player,
    /// Depth of the current search (set at search entry).
    search_depth: u32,
    /// Extensions used on the current path (reset per root move).
    extensions_used: u32,
}

impl<E: Evaluator> OppsSearcher<E> {
    /// Create a new OPPS searcher with default configuration and 16MB TT.
    pub fn new(evaluator: E) -> Self {
        Self {
            config: OppsConfig::default(),
            evaluator,
            tt: TranspositionTable::new(16),
            killers: KillerTable::new(),
            history: HistoryTable::new(),
            swarm: SwarmPipeline::new_stub(),
            nodes: 0,
            root_player: Player::Red,
            search_depth: 0,
            extensions_used: 0,
        }
    }

    /// Create with custom config and TT size.
    pub fn with_config(evaluator: E, config: OppsConfig, tt_mb: usize) -> Self {
        Self {
            config,
            evaluator,
            tt: TranspositionTable::new(tt_mb),
            killers: KillerTable::new(),
            history: HistoryTable::new(),
            swarm: SwarmPipeline::new_stub(),
            nodes: 0,
            root_player: Player::Red,
            search_depth: 0,
            extensions_used: 0,
        }
    }

    /// The core alpha-beta search.
    ///
    /// `depth`: remaining depth to search (counts down to 0).
    /// `ply`: distance from root (counts up from 0).
    /// `alpha`/`beta`: pruning bounds from root player's perspective (via negamax).
    fn alpha_beta(
        &mut self,
        state: &mut GameState,
        depth: u32,
        ply: usize,
        mut alpha: Score,
        beta: Score,
    ) -> Score {
        self.nodes += 1;

        // Terminal: game over
        if state.is_game_over() {
            // If root player is the last one standing, that's a win
            if state.is_active(self.root_player) {
                return SCORE_MATE - ply as Score;
            }
            return -SCORE_MATE + ply as Score;
        }

        // Leaf: evaluate via swarm with stability-driven extensions
        if depth == 0 {
            let assessment = self.swarm.assess(state, &self.evaluator);

            // If position is unstable and we have extension budget, extend by one round
            if assessment.stability < self.config.stability_threshold
                && self.extensions_used < self.config.max_extensions
            {
                self.extensions_used += 1;
                let extended_score = self.alpha_beta(state, 4, ply, alpha, beta);
                self.extensions_used -= 1;
                return extended_score;
            }

            return assessment.score;
        }

        // TT probe
        let tt_move = self.tt.probe_move(state.zobrist_hash);
        if let Some(entry) = self.tt.probe(state.zobrist_hash, depth) {
            match entry.flag {
                TTFlag::Exact => return entry.score,
                TTFlag::LowerBound => {
                    if entry.score >= beta {
                        return entry.score;
                    }
                }
                TTFlag::UpperBound => {
                    if entry.score <= alpha {
                        return entry.score;
                    }
                }
            }
        }

        // Generate legal moves
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        generate_legal_moves(state, &mut moves);

        // No legal moves = stalemate or checkmate
        if moves.is_empty() {
            // In 4PC, if side_to_move has no legal moves, they are eliminated.
            // From paranoid perspective: if it's the root player, bad. If opponent, good.
            if state.side_to_move() == self.root_player {
                return -SCORE_MATE + ply as Score;
            }
            return SCORE_DRAW;
        }

        // Order moves
        let scored_moves = score_moves(&moves, tt_move, &self.killers, &self.history, ply);

        // Determine move limit for opponent pruning
        let is_root_player = state.side_to_move() == self.root_player;
        let move_limit = if is_root_player {
            scored_moves.len() // Root player: consider all moves
        } else {
            // Opponent: limit based on OPPS parameters
            let plies_from_root = ply;
            if plies_from_root <= 4 {
                self.config.l1.min(scored_moves.len())
            } else {
                self.config.l2.min(scored_moves.len())
            }
        };

        let original_alpha = alpha;
        let mut best_score = SCORE_NEG_INFINITY;
        let mut best_move = Move::NULL;

        for (i, sm) in scored_moves.iter().enumerate() {
            if i >= move_limit {
                break;
            }

            let mv = sm.mv;
            let undo = state.make_move(mv);
            let score = -self.alpha_beta(state, depth - 1, ply + 1, -beta, -alpha);
            state.unmake_move(mv, undo);

            if score > best_score {
                best_score = score;
                best_move = mv;
            }

            if score > alpha {
                alpha = score;
            }

            if alpha >= beta {
                // Beta cutoff
                if !mv.is_capture() {
                    self.killers.store(ply, mv);
                    self.history.update(mv.from.index(), mv.to.index(), depth);
                }
                break;
            }
        }

        // TT store (only at depth-4-aligned depths)
        if depth.is_multiple_of(4) {
            let flag = if best_score <= original_alpha {
                TTFlag::UpperBound
            } else if best_score >= beta {
                TTFlag::LowerBound
            } else {
                TTFlag::Exact
            };
            self.tt
                .store(state.zobrist_hash, best_move, best_score, depth, flag);
        }

        best_score
    }
}

impl<E: Evaluator> Searcher for OppsSearcher<E> {
    fn search(&mut self, state: &GameState, depth: u32) -> SearchResult {
        assert!(
            depth.is_multiple_of(4) && depth > 0,
            "OPPS search depth must be a positive multiple of 4, got {}",
            depth
        );

        // Setup for new search
        self.root_player = state.side_to_move();
        self.search_depth = depth;
        self.nodes = 0;
        self.extensions_used = 0;
        self.killers.clear();
        self.tt.new_search();

        // Clone state for mutable search
        let mut search_state = state.clone();

        // Generate root moves
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        generate_legal_moves(&search_state, &mut moves);

        if moves.is_empty() {
            return SearchResult {
                best_move: Move::NULL,
                score: -SCORE_MATE,
                depth,
                nodes: 1,
            };
        }

        // Order root moves
        let tt_move = self.tt.probe_move(search_state.zobrist_hash);
        let scored_moves = score_moves(&moves, tt_move, &self.killers, &self.history, 0);

        let mut best_score = SCORE_NEG_INFINITY;
        let mut best_move = scored_moves[0].mv;
        let mut alpha = SCORE_NEG_INFINITY;
        let beta = crate::types::SCORE_INFINITY;

        for sm in scored_moves.iter() {
            let mv = sm.mv;
            let undo = search_state.make_move(mv);
            let score = -self.alpha_beta(&mut search_state, depth - 1, 1, -beta, -alpha);
            search_state.unmake_move(mv, undo);

            if score > best_score {
                best_score = score;
                best_move = mv;
            }

            if score > alpha {
                alpha = score;
            }
        }

        // Store root position in TT
        if depth.is_multiple_of(4) {
            self.tt.store(
                state.zobrist_hash,
                best_move,
                best_score,
                depth,
                TTFlag::Exact,
            );
        }

        SearchResult {
            best_move,
            score: best_score,
            depth,
            nodes: self.nodes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::eval::BootstrapEvaluator;
    use crate::game_state::GameState;

    fn make_searcher() -> OppsSearcher<BootstrapEvaluator> {
        OppsSearcher::new(BootstrapEvaluator::new())
    }

    #[test]
    #[should_panic(expected = "positive multiple of 4")]
    fn test_rejects_depth_3() {
        let state = GameState::new();
        let mut searcher = make_searcher();
        searcher.search(&state, 3);
    }

    #[test]
    #[should_panic(expected = "positive multiple of 4")]
    fn test_rejects_depth_0() {
        let state = GameState::new();
        let mut searcher = make_searcher();
        searcher.search(&state, 0);
    }

    #[test]
    #[should_panic(expected = "positive multiple of 4")]
    fn test_rejects_depth_5() {
        let state = GameState::new();
        let mut searcher = make_searcher();
        searcher.search(&state, 5);
    }

    #[test]
    fn test_depth_4_returns_legal_move() {
        let state = GameState::new();
        let mut searcher = make_searcher();
        let result = searcher.search(&state, 4);

        // Must return a legal move
        let mut moves = ArrayVec::<Move, MAX_MOVES>::new();
        generate_legal_moves(&state, &mut moves);
        assert!(
            moves.iter().any(|m| *m == result.best_move),
            "Search returned illegal move"
        );

        // Must have searched some nodes
        assert!(result.nodes > 0);
        assert_eq!(result.depth, 4);
    }

    #[test]
    fn test_depth_4_score_reasonable() {
        let state = GameState::new();
        let mut searcher = make_searcher();
        let result = searcher.search(&state, 4);

        // Starting position should be roughly equal (within 200cp)
        assert!(
            result.score.abs() < 200,
            "Starting position score {} is unreasonable",
            result.score
        );
    }

    #[test]
    fn test_depth_8_deeper_search() {
        let state = GameState::new();
        let mut searcher = make_searcher();
        let result_4 = searcher.search(&state, 4);
        let result_8 = searcher.search(&state, 8);

        // Depth 8 should search more nodes than depth 4
        assert!(
            result_8.nodes > result_4.nodes,
            "depth 8 ({} nodes) should search more than depth 4 ({} nodes)",
            result_8.nodes,
            result_4.nodes
        );
    }

    #[test]
    fn test_tt_reduces_nodes() {
        let state = GameState::new();

        // First search populates TT
        let mut searcher = make_searcher();
        let result1 = searcher.search(&state, 4);

        // Second search with warm TT should visit fewer or equal nodes
        let result2 = searcher.search(&state, 4);

        // TT should help (or at least not hurt)
        assert!(
            result2.nodes <= result1.nodes + 10,
            "Second search ({} nodes) should benefit from TT (first: {})",
            result2.nodes,
            result1.nodes
        );
    }

    #[test]
    fn test_search_visits_nonzero_nodes() {
        let state = GameState::new();
        let mut searcher = make_searcher();
        let result = searcher.search(&state, 4);
        assert!(
            result.nodes > 100,
            "Search should visit many nodes at depth 4"
        );
    }

    #[test]
    fn test_extension_with_full_swarm() {
        // Full swarm with moderate threshold — extensions may trigger on contested positions
        let state = GameState::new();
        let mut searcher = make_searcher();
        searcher.swarm = crate::swarm::SwarmPipeline::new();
        searcher.config.stability_threshold = 0.3; // moderate: only extend very unstable
        searcher.config.max_extensions = 1; // limit to 1 round of extensions

        let result = searcher.search(&state, 4);
        assert!(!result.best_move.is_null());
        assert!(result.nodes > 0);
    }

    #[test]
    fn test_stub_swarm_no_extensions() {
        // Stub swarm always returns stability=1.0, so no extensions should trigger
        let state = GameState::new();

        let mut searcher_a = make_searcher();
        searcher_a.config.stability_threshold = 0.5;
        searcher_a.config.max_extensions = 2;
        let nodes_with_ext = searcher_a.search(&state, 4).nodes;

        let mut searcher_b = make_searcher();
        searcher_b.config.stability_threshold = 0.5;
        searcher_b.config.max_extensions = 0;
        let nodes_no_ext = searcher_b.search(&state, 4).nodes;

        // With stub swarm (stability=1.0), extensions never trigger
        assert_eq!(
            nodes_with_ext, nodes_no_ext,
            "Stub swarm should never trigger extensions"
        );
    }

    #[test]
    fn test_self_play_10_moves() {
        let mut state = GameState::new();
        let mut searcher = make_searcher();

        // Play 10 moves (not 10 rounds — 10 individual ply)
        for i in 0..10 {
            if state.is_game_over() {
                break;
            }

            let result = searcher.search(&state, 4);
            assert!(
                !result.best_move.is_null(),
                "Search returned null move at ply {}",
                i
            );

            // Verify the move is legal
            let mut legal = ArrayVec::<Move, MAX_MOVES>::new();
            generate_legal_moves(&state, &mut legal);
            assert!(
                legal.iter().any(|m| *m == result.best_move),
                "Illegal move at ply {}: {:?}",
                i,
                result.best_move
            );

            state.make_move(result.best_move);
        }
    }
}
