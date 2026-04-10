# Architectural Decision Records — Project Valhalla

---

## ADR-001: Board Abstraction via Attack Query API

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** Carried from Odin v1

**Decision:** Nothing above Phase 1 reads `board.squares[]` directly. All piece queries go through the Attack Query API (`is_attacked_by`, `attackers_of`, `pieces_for_player`, etc.).

**Rationale:** Direct array access creates tight coupling between board representation and every consumer. The API allows board internals to change without cascading rewrites.

**Consequences:**
- Phase 1 must define and stabilize the API before Phase 2 begins.
- Eval, search, and swarm layers depend only on the API, not on board layout.
- Slight indirection cost, negligible in practice.

---

## ADR-002: OPPS Search (Supersedes BRS/Paranoid Hybrid)

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** New for Valhalla

**Decision:** Valhalla uses OPPS (Opponent Pruning Paranoid Search) from Baier & Kaisers (2020) as its primary tree search. OPPS generalizes paranoid search with three parameters: n1 (number of opponent moves at depth 1), l1 (opponent moves at subsequent depths), and l2 (opponent moves at the deepest level). BRS+ is the special case OPPS(1, infinity, 1). Valhalla does not use the BRS/Paranoid hybrid from Odin v1.

**Rationale:** OPPS provides a single tunable framework that subsumes paranoid, BRS, and intermediate strategies. The n1/l1/l2 parameters allow runtime tuning between full paranoid (exhaustive opponent moves) and aggressive pruning (single best-reply). This is strictly more flexible than the hardcoded BRS/Paranoid toggle in Odin v1.

**Consequences:**
- Search must accept n1/l1/l2 configuration.
- Move ordering quality directly affects OPPS performance (bad ordering with low l1 = missed moves).
- Tuning n1/l1/l2 becomes part of Phase 3 acceptance criteria.

---

## ADR-003: Evaluator Trait (Single Interface)

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** New for Valhalla

**Decision:** The evaluator trait exposes a single method: `fn evaluate(&self, state: &GameState) -> Score`. Bootstrap eval implements it first; NNUE replaces it as a drop-in. Output is a 1-perspective scalar relative to the searching player.

**Rationale:** A single `evaluate()` interface keeps consumers simple. The 1-perspective scalar avoids the complexity of Freyja's 4-vector approach (`eval_4vec`), which required Max^n to handle four independent scores. OPPS is paranoid-family search and only needs the root player's perspective.

**Consequences:**
- No `eval_scalar` / `eval_4vec` split.
- NNUE training targets are single scalars, simplifying the training pipeline.
- If future search needs opponent evals, they call `evaluate()` from each player's perspective (score negation for paranoid assumption).

---

## ADR-004: Fixed-Size Data Structures

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** Carried from Odin v1

**Decision:** No `Vec<T>` in Board, GameState, or MoveUndo. All hot-path data uses fixed-size arrays or ArrayVec.

**Rationale:** Heap allocation in inner loops destroys cache performance. The 14x14 board has bounded dimensions; all structures can be statically sized.

**Consequences:**
- Maximum piece counts, move lists, and undo stacks must have compile-time bounds.
- ArrayVec<Move, 256> for move generation (see ADR-013).

---

## ADR-005: Zobrist Hash Includes Root Player

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** Carried from Odin v1

**Decision:** The Zobrist hash incorporates the root player (the player whose perspective the search is from), not just the side to move.

**Rationale:** In 4-player chess, the same board position with a different root player is a fundamentally different search node. TT entries must distinguish these to avoid returning scores computed from the wrong perspective.

**Consequences:**
- TT lookups require root player as part of the key.
- Hash table entries are not reusable across different root-player searches of the same position.

---

## ADR-006: tracing Crate (No Custom Telemetry)

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** Carried from Odin v1

**Decision:** Use the `tracing` crate with `tracing-subscriber` for all logging and diagnostics. No custom telemetry frameworks.

**Rationale:** `tracing` is the Rust ecosystem standard. Structured spans and events, compile-time filtering, zero-cost when disabled. Rolling a custom system wastes time and produces worse results.

**Consequences:**
- All modules use `tracing::{info, debug, warn, error, trace}`.
- `RUST_LOG` environment variable controls verbosity.
- Subscriber initialization in main.rs.

---

## ADR-007: Observer Protocol from Phase 2

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** Carried from Odin v1 (updated)

**Decision:** The observer (external test harness, eval tuning suite, game replay) is available starting Phase 2, as soon as search can produce moves.

**Rationale:** Early observability catches eval and search bugs before they compound. Phase 2 delivers bootstrap eval + influence maps, which is the minimum needed for meaningful observer output.

**Consequences:**
- Observer infrastructure (JSON configs, run scripts) must be ready by end of Phase 2.
- Phase 1 (board + rules) has no observer — tested via unit tests only.

---

## ADR-008: Eval/Search/Swarm Separation

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** Carried from Odin v1 (updated)

**Decision:** Tactical phenomena (captures, checks, exchanges, pins, forks) belong in the Swarm layer (see ADR-018). Strategic features (material, PST, mobility, territory, king safety) belong in eval. Search orchestrates both but contains neither.

**Rationale:** Mixing tactical resolution into search (quiescence) is a 2-player technique that scales poorly to 4 players. Swarm provides structured tactical resolution as a pipeline between search and eval. Keeping eval purely strategic simplifies NNUE training targets.

**Consequences:**
- Eval never reasons about move sequences — only static features.
- Swarm handles all tactical continuation logic.
- Search calls swarm at leaf nodes instead of quiescence search.

---

## ADR-009: No Lead Penalty in Eval

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** Carried from Odin v1

**Decision:** The evaluation function does not penalize the leading player. Each player's eval is independent — no "tall poppy" adjustment.

**Rationale:** Lead penalties create bizarre incentives where the engine deliberately plays worse to avoid being targeted. In OPPS (paranoid family), the search already assumes opponents gang up on the root player. Adding an eval penalty on top double-counts the threat.

**Consequences:**
- Players with large advantages get large eval scores.
- Gang-up dynamics are handled by search (OPPS paranoid assumption), not eval.

---

## ADR-010: Gumbel MCTS over UCB1

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** From Freyja ADR-006

**Decision:** MCTS uses Gumbel-Top-k sampling (Danihelka et al., 2022) instead of UCB1 for action selection at root.

**Rationale:** UCB1 wastes simulations exploring clearly bad moves. Gumbel sampling uses the policy prior to focus simulations on the top-k most promising moves, achieving better move quality with fewer simulations. Critical for 4-player chess where the branching factor is enormous.

**Consequences:**
- Requires a policy prior (from NNUE or OPPS move ordering).
- Top-k parameter controls exploration breadth.
- Sequential halving reduces simulation budget further.

---

## ADR-011: Progressive History (OPPS -> MCTS)

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** Carried from Odin v1 (updated)

**Decision:** The history heuristic table built during OPPS search is extracted and injected into MCTS as a prior for move ordering.

**Rationale:** OPPS explores a focused subtree deeply. The resulting history scores encode which moves were good in practice. Feeding this into MCTS avoids cold-starting the tree policy and improves early simulation quality.

**Consequences:**
- History table format must be compatible between OPPS and MCTS.
- OPPS must run before MCTS in the phase-separated hybrid (see ADR-020).

---

## ADR-012: EP Board Scan

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** Carried from Odin v1

**Decision:** En passant detection uses board scanning (`find_ep_captured_pawn_sq()`) rather than `player.prev()`.

**Rationale:** In 4-player chess, `player.prev()` returns eliminated players when players have been checkmated. Board scanning for the en passant target square is always correct regardless of elimination state.

**Consequences:**
- EP movegen scans for the target pawn on the board.
- Slightly more work per EP check, but correctness is non-negotiable.

---

## ADR-013: ArrayVec Movegen

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** Carried from Odin v1

**Decision:** Move generation writes into `ArrayVec<Move, 256>` via a MoveBuffer trait. No heap allocation in movegen.

**Rationale:** The maximum legal moves in 4-player chess on a 14x14 board is bounded. ArrayVec provides Vec-like ergonomics with stack allocation. 256 is generous headroom.

**Consequences:**
- MoveBuffer trait allows different consumers (search, perft, test) to use the same movegen.
- If 256 is ever insufficient, it's a compile-time constant to bump.

---

## ADR-014: SIMD-Ready NNUE

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** Carried from Odin v1

**Decision:** NNUE implementation plans for SIMD from day one: weight transpose at load time, `align(32)` accumulators, runtime AVX2/SSE detection.

**Rationale:** Retrofitting SIMD into an NNUE that wasn't designed for it requires rewriting the forward pass and weight layout. Planning alignment and data layout up front makes SIMD a drop-in optimization.

**Consequences:**
- Weight files store in SIMD-friendly layout.
- Accumulator arrays are 32-byte aligned.
- Fallback scalar path for CPUs without AVX2.

---

## ADR-015: Release Profile from Phase 0

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** Carried from Odin v1 (updated)

**Decision:** LTO (thin) and `codegen-units = 1` enabled in the release profile from Phase 0 onward.

**Rationale:** Free 10-20% performance improvement with no code changes. Setting it up from the start means all benchmarks reflect production performance.

**Consequences:**
- Release builds are slower to compile.
- Debug builds remain fast (default profile unchanged).

---

## ADR-016: Two-File Agent Governance

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** Carried from Odin v1

**Decision:** Agent behavior is governed by two files: AGENT_CONDUCT.md (standard operations) and AGENT_CONDUCT_MYTHOS.md (narrative variant with identical rules). Agents choose which to follow based on configuration.

**Rationale:** The Mythos variant provides the same operational rules in a different framing that some agent configurations respond to more reliably. Both files must stay in sync.

**Consequences:**
- Any rule change must be applied to both files.
- CLAUDE.md points to both as authoritative.

---

## ADR-017: Strategy Profiles as Two Axes

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** Carried from Odin v1

**Decision:** Training data diversity uses strategy profiles with two independent axes: target selection (who to attack) and play style (how to play). These are orthogonal — any target strategy can combine with any play style.

**Rationale:** Coupling target selection with play style (e.g., "aggressive = attack leader") reduces training diversity. Independent axes produce N*M combinations from N+M definitions.

**Consequences:**
- Self-play games sample from both axes independently.
- Each axis has 3-5 profiles (e.g., target: leader/weakest/neighbor/random; style: aggressive/positional/defensive/chaotic).

---

## ADR-018: Swarm Replaces Quiescence Search

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** New for Valhalla

**Decision:** Valhalla uses a six-layer Swarm pipeline instead of quiescence search for tactical resolution at leaf nodes. The six layers are:
1. **Force Ratio** — Material balance and threats in a zone.
2. **Pile-On** — Multiple attackers converging on one target.
3. **Chain Walk** — Capture chains (A takes B, C takes A, D takes C...).
4. **Swarm-Delta** — Net material change from the full exchange sequence.
5. **Commitment Counting** — How many pieces are committed to the fight.
6. **Chain Participation** — Which players are involved and who benefits.

**Rationale:** Quiescence search (stand-pat + capture extension) is a 2-player technique. In 4-player chess, captures involve up to 4 players with shifting alliances. A structured pipeline that analyzes the tactical situation as a whole produces better results than recursively extending captures. Research backing: SUPER-SOMA (multi-agent tactical resolution), EBQS (extended best-first quiescence).

**Consequences:**
- No quiescence search in the codebase.
- Swarm is co-developed with OPPS in Phase 3.
- Each swarm layer is independently testable.
- Swarm output feeds into eval as tactical adjustment.

---

## ADR-019: DKW Rules

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** From Freyja (proven design)

**Decision:** Dead King Walking (DKW): when a player is checkmated, their pieces become immovable walls and their king moves randomly (or not at all, configurable). Eliminated players do not take turns in the move order.

**Rationale:** Several alternatives exist for eliminated players (pieces removed, pieces become neutral, pieces frozen). DKW with frozen pieces and skipped turns is the simplest correct implementation. Immovable walls create interesting board dynamics. Freyja proved this design works in practice.

**Consequences:**
- Move generation must skip eliminated players.
- Board must track elimination status per player.
- Eval must account for wall pieces (they block movement but cannot be captured for material).

---

## ADR-020: Phase-Separated Hybrid

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** From Freyja ADR-017

**Decision:** Opening phase uses OPPS only; midgame switches to MCTS (with OPPS rollouts). The cutover ply is configurable (default: ply 40).

**Rationale:** OPPS is faster per node and sufficient for opening development where tactics are sparse. MCTS excels in complex middlegame positions where the branching factor explodes and strategic planning matters more than tactical depth. The hybrid avoids paying MCTS overhead in simple positions.

**Consequences:**
- Engine must detect phase transition (configurable ply threshold).
- OPPS history feeds into MCTS prior (see ADR-011).
- Both search modes must use the same evaluator trait.

---

## ADR-021: OMA + Progressive Widening in MCTS

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** From Freyja ADR-018/019

**Decision:** MCTS uses two techniques to handle the 4-player branching factor:
- **OMA (One Move Approximation):** At opponent nodes, pick a single move via lightweight policy instead of expanding all children. This makes the effective tree 3-4x deeper for the same simulation count.
- **Progressive Widening (PW):** At root-player nodes, expand floor(k * visits^alpha) children. New moves are added as the node accumulates visits, focusing early simulations on the most promising moves.

**Rationale:** Full expansion at every node in 4-player MCTS is intractable (branching factor ~120^4 per round). OMA reduces opponent branching to 1, and PW reduces root-player branching to a growing subset. Together they make MCTS viable for 4-player chess.

**Consequences:**
- OMA requires a fast policy for opponent moves (history heuristic or lightweight NNUE).
- PW parameters (k, alpha) need tuning.
- Move ordering quality directly impacts OMA accuracy.

---

## ADR-022: Ray-Attenuated Influence Maps

**Status:** Accepted
**Date:** 2026-04-10
**Origin:** From Freyja ADR-020

**Decision:** Influence maps use per-piece ray casting along movement vectors with blocker attenuation. Each piece projects influence along its legal movement directions; influence decays with distance and is blocked (attenuated) by intervening pieces.

**Rationale:** Simple BFS/Voronoi influence maps treat all squares equally regardless of piece placement. Ray attenuation captures the fact that a bishop behind a pawn wall has less influence than one on an open diagonal. This produces more accurate territory and king safety assessments.

**Consequences:**
- Influence computation is more expensive than simple Voronoi.
- Sliding pieces (bishop, rook, queen) benefit most from ray attenuation.
- Influence maps are recomputed incrementally where possible (accumulator updates).
