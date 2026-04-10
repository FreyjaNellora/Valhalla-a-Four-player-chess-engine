# Project Valhalla — Masterplan

**A novel four-player chess engine built from first principles**

Version 2.0 — April 2026

---

## 1. Vision and Thesis

Project Valhalla is a four-player chess engine designed from scratch around the structural reality that four-player search is a fundamentally different problem — one where deep pruning is provably limited (Korf 1991), where search pathology is a real threat (Nau 1982–83), and where tactical resolution cannot be delegated to quiescence search because three opponents can intervene between any two of your moves.

The engine is built on four interlocking ideas:

**OPPS (Opponent-Pruning Paranoid Search)** generalizes paranoid search with tunable parameters that control how aggressively opponent subtrees are pruned. Rather than assuming all opponents play optimally against you (pure paranoid) or expanding all opponent moves (Max^n), OPPS lets the engine dial between these extremes based on what the position demands. The parameters — node budget n1, paranoid depth limit l1, and opponent pruning depth l2 — create a search that is paranoid where it needs to be and efficient where it can afford to be. OPPS supersedes the BRS/Paranoid hybrid from Odin v1 (ADR-002). BRS+ is a special case of OPPS: OPPS(1, ∞, 1). (Baier & Kaisers 2020)

**Swarm Tactical Resolution** replaces quiescence search entirely. In two-player chess, quiescence works because you alternate moves with one opponent — you can extend captures until the position is quiet. In four-player chess, "quiet" is nearly meaningless: three other players move before you get another turn. Swarm is a six-layer pipeline that runs at every leaf node: Force Ratio → Pile-On Detection → Chain Walk → Swarm-Delta → Commitment Counting → Chain Participation. Each layer feeds the next. The pipeline determines both a tactical evaluation and a stability signal that tells OPPS whether this leaf is resolved enough to trust. Research precedent: SUPER-SOMA (Rollason 2000, branchless capture resolution), EBQS (Schadd & Winands 2009, static eval replacing qsearch).

**MCTS as strategic planning layer** sits above OPPS+Swarm. MCTS handles the strategic question (which candidate move looks best given uncertain multi-agent dynamics) while OPPS+Swarm handles the tactical question (what is the concrete outcome of a given line). Valhalla uses Gumbel-Top-k selection with Sequential Halving at the root (proven in Freyja to converge in 2 simulations vs 16+ for UCB1), Opponent Move Abstraction for 3-4x deeper MCTS simulations, and Progressive Widening at root-player nodes. A phase-separated hybrid controller runs OPPS alone in structured openings and switches to MCTS in chaotic midgame positions.

**1-Perspective NNUE** uses a single accumulator with scalar output and relative encoding. The active player's perspective is always the one encoded; other perspectives are obtained by rotating the board at simulation time. This is the correct choice for OPPS — paranoid search only needs the root player's score, so a single-perspective network suffices. Training uses the generational self-play methodology: self-play → accumulate data → warm-start from previous generation → export weights → repeat.

### What makes this different

The central architectural commitment is **co-development of search and evaluation from day one**, following Generalized Policy Iteration (AlphaZero/ExIt). The bootstrap evaluator is weak but real — a purely positional assessor with zero tactical terms, because swarm handles all tactical assessment. Search and evaluation are born together.

The second commitment is that **swarm is part of search, not a post-processing step**. The swarm stability signal determines whether a node is a leaf at all. Swarm and OPPS are coupled from their first line of code.

The third commitment is that the build order follows from actual dependency chains, not pedagogical convenience.

### Heritage

Valhalla inherits engineering lessons from two predecessor engines:

**From Odin v1 (Stages 0-19):** Fixed-size data structures, Attack Query API, player-aware TT, EP board scan, ArrayVec movegen, LTO+codegen-units=1, no bitboards on 14x14, stress test volume over depth, strategy profiles (vulture/predator/assassin × fortress/territorial), no lead penalty in eval.

**From Freyja (Stages 0-17):** Gumbel-Top-k MCTS, Opponent Move Abstraction, Progressive Widening, phase-separated hybrid, DKW rules, ray-attenuated influence maps, observer protocol for self-play and A/B testing, chain walk design (multiplayer SEE), commitment risk evaluation. Freyja also demonstrated that Max^n is impractical for deep search in 4-player chess due to branching factor — validating the OPPS approach.

**NNUE generational training** (warm-start, EMA, accumulating data across generations) is currently under test in Freyja. Results are pending — the methodology is not yet proven.

---

## 2. Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│              Phase-Separated Hybrid Controller                      │
│  Opening (ply < cutover): OPPS only, full time budget               │
│  Midgame (ply ≥ cutover): MCTS with OPPS simulations               │
│                                                                     │
│  MCTS Strategic Layer (midgame only):                               │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  Gumbel-Top-k root selection + Sequential Halving               │ │
│  │  Progressive Widening at root-player nodes: floor(k*visits^α)   │ │
│  │  OMA at opponent nodes: 1 move via lightweight policy           │ │
│  │  Opponent model: 70% paranoid / 20% max-own / 10% random       │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  Each simulation (or standalone in opening):                        │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │                 OPPS Tactical Search                             │ │
│  │  Paranoid backbone with parameterized opponent pruning           │ │
│  │  Parameters: n1 (node budget), l1 (paranoid limit),             │ │
│  │              l2 (opponent prune depth)                           │ │
│  │  Depth rule: only d ≡ 0 (mod 4) are valid terminal depths      │ │
│  │  Move ordering: TT → killer → history → MVV-LVA                │ │
│  │                                                                  │ │
│  │  At each leaf:                                                   │ │
│  │  ┌──────────────────────────────────────────────────────────────┐│ │
│  │  │          Swarm Tactical Resolution (6 layers)                ││ │
│  │  │                                                              ││ │
│  │  │  L1: Force Ratio — contested squares, balance of power       ││ │
│  │  │  L2: Pile-On Detection — lurkers vs committed                ││ │
│  │  │  L3: Chain Walk — resolve capture sequences (branchless)     ││ │
│  │  │  L4: Swarm-Delta — cost of engaging elsewhere                ││ │
│  │  │  L5: Commitment Counting — overextension across fronts       ││ │
│  │  │  L6: Chain Participation — initiate, reinforce, or stay out  ││ │
│  │  │                                                              ││ │
│  │  │  Output: (score: i32, stability: f32)                        ││ │
│  │  │  stability < threshold → request deeper search               ││ │
│  │  └──────────────────────────────────────────────────────────────┘│ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  Evaluator (called by swarm as base signal):                        │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  Phase 1-3: Bootstrap Evaluator (positional only)               │ │
│  │    material, PST, king safety (structural), pawn structure      │ │
│  │    NO tactical terms — swarm handles all tactics                │ │
│  │                                                                  │ │
│  │  Phase 5+:  1-Perspective NNUE                                  │ │
│  │    single accumulator, scalar output, relative encoding         │ │
│  │    other perspectives via 90°/180°/270° board rotation          │ │
│  └─────────────────────────────────────────────────────────────────┘ │
│                                                                     │
│  Influence Infrastructure (feeds swarm + eval):                     │
│  ┌─────────────────────────────────────────────────────────────────┐ │
│  │  Ray-attenuated influence maps (per-piece, directional)         │ │
│  │    Rook/Bishop/Queen: ray-walk with blocker attenuation         │ │
│  │    Knight/Pawn/King: direct projection to attack squares        │ │
│  └─────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│                         Infrastructure                               │
│                                                                     │
│  Board (14×14, fixed-size arrays, no Vec<T> on hot path)            │
│  GameState (turn order R→B→Y→G, move generation, perft-validated)   │
│  DKW (Dead King Walking: pieces as walls, king moves randomly)      │
│  MoveUndo (fixed-size, stack-allocated)                             │
│  Zobrist hashing (4-player aware, incremental update)               │
│  Transposition table (depth-4 aware replacement policy)             │
│                                                                     │
│  Frozen traits:                                                     │
│    trait Searcher { fn search(&mut self, state, depth) -> Move; }   │
│    trait Evaluator { fn evaluate(&self, state) -> Score; }          │
│                                                                     │
│  Tech stack:                                                        │
│    Engine: Rust │ UI: TypeScript + React + Tauri                    │
│    Training: Python + PyTorch │ Observer: Node.js                   │
└─────────────────────────────────────────────────────────────────────┘
```

### Data flow

1. **Hybrid controller** determines mode: OPPS-only (opening) or MCTS (midgame).
2. In MCTS mode, **Gumbel-Top-k** selects a candidate move. **OMA** collapses opponent nodes to 1 move. **Progressive Widening** limits root-player breadth.
3. For each simulation (or standalone search), **OPPS** searches the resulting position to a depth divisible by 4.
4. At every leaf, **Swarm** runs its six-layer pipeline using the current **Evaluator** and **influence maps** as base signal.
5. Swarm returns `(score, stability)`. If stability is below threshold and depth budget remains, OPPS extends by one full round (4 plies).
6. When time/node budget is exhausted, the controller returns the best move.
7. In 2-player endgame (2 active players + DKW pieces), search switches to **negamax with expectiminimax** for DKW chance nodes.

### Key interfaces (frozen once defined)

```rust
pub trait Evaluator: Send + Sync {
    /// Pure positional assessment. No tactical terms.
    /// Returns score from the perspective of state.side_to_move().
    fn evaluate(&self, state: &GameState) -> Score;
}

pub trait Searcher {
    /// Returns the best move for the current side to move.
    /// depth must be divisible by 4.
    fn search(&mut self, state: &GameState, depth: u32) -> SearchResult;
}

pub struct SwarmAssessment {
    pub score: Score,
    pub stability: f32,       // 0.0 = chaotic, 1.0 = fully resolved
    pub layer_scores: [f32; 6], // per-layer breakdown for diagnostics
}

pub struct SearchResult {
    pub best_move: Move,
    pub score: Score,
    pub depth: u32,
    pub nodes: u64,
}
```

---

## 3. Build Phases

### Phase 1: Foundation — Board, Rules, Move Generation, and DKW

**What you're building:** The complete board representation, game state, legal move generation, make/unmake, Zobrist hashing, and Dead King Walking rules for 14×14 four-player chess. Fixed-size data structures throughout. Perft validation.

**Why it's needed now:** Everything depends on a correct and fast board. OPPS cannot search without move generation. Swarm cannot assess without board state.

**Depends on:** Nothing. This is the root of the dependency chain.

**What's in scope:**

The board is 14×14 with corner dead zones (3×3 corners not playable). Four players — Red, Blue, Yellow, Green — with turn order R→B→Y→G. Each player starts with 16 pieces on their respective edges.

Board representation uses fixed-size arrays. `[Piece; 196]` for the mailbox. No `Vec<T>` anywhere in Board, GameState, or MoveUndo. The hot path is allocation-free.

Move generation covers all standard chess moves adapted for four players: pawn pushes (direction depends on player), captures (including en passant across four directions), castling per player, piece moves. Promotion when a pawn reaches the opposite edge. En passant uses board scan (`find_ep_captured_pawn_sq()`), NOT `player.prev()` — which returns eliminated players in 4PC.

MoveUndo is a fixed-size struct. Stack-allocated, no heap.

Zobrist hashing is four-player aware: piece-square keys for all four colors, side-to-move keys, castling right keys, en passant file keys per player direction. Incremental update on make/unmake.

**DKW (Dead King Walking):** When a player is eliminated, their pieces become immovable, uncapturable walls. Their king remains live and moves randomly (1-8 possible moves per turn). DKW state tracked via flags on GameState. This is a game rule requirement for any engine supporting FFA with elimination.

**Acceptance criteria:**

- [ ] `perft(1) = 20`, `perft(2) = 395`, `perft(3) = 7,800`, `perft(4) = 152,050`
- [ ] make/unmake round-trip preserves identical GameState including Zobrist hash
- [ ] No `Vec<T>`, `Box<T>`, or heap allocation in Board, GameState, MoveUndo
- [ ] Move generation correct for all four player directions (pawns, castling, EP)
- [ ] Dead zone squares never appear in any generated move list
- [ ] Zobrist hash collision rate < expected for hash width (measured on 1M+ positions)
- [ ] DKW pieces are immovable walls, DKW king generates 1-8 random moves
- [ ] EP uses board scan, not player.prev()

---

### Phase 2: Bootstrap Evaluator, Influence Maps, and Observer

**What you're building:** The Evaluator and Searcher trait definitions (frozen once defined), the bootstrap evaluator (purely positional), ray-attenuated influence maps, and the observer protocol for self-play and testing.

**Why it's needed now:** OPPS and swarm both need an evaluator. The traits are the frozen interface contract. Influence maps are infrastructure that swarm layers consume — building them now means swarm can use them from day one. The observer is needed as soon as the engine can play moves (Phase 3), so the infrastructure must exist before then.

**Depends on:** Phase 1 (Board, GameState).

**What's in scope:**

**Evaluator trait:** `fn evaluate(&self, state: &GameState) -> Score`. Pure function of position. No search context.

**Searcher trait:** `fn search(&mut self, state: &GameState, depth: u32) -> SearchResult`. Depth must be divisible by 4. `&mut self` allows internal state (TT, statistics).

**Bootstrap evaluator** implements `Evaluator` with four components:

1. **Material balance.** Piece values (P=100, N=320, B=330, R=500, Q=900, K=∞). Your material minus average of opponents' material.
2. **Piece placement.** Piece-square tables for all four orientations. Precomputed by rotating base tables.
3. **King safety (structural).** Pawn shield integrity, open files near king, castling status. No tactical threats — swarm's domain.
4. **Pawn structure.** Doubled, isolated, passed pawns, pawn chains. Four directions.

No tactical terms. No fork detection, pin detection, hanging piece detection. Those are swarm's domain. This separation is a core architectural decision.

**Ray-attenuated influence maps:** Per-piece influence projected along actual movement vectors.
- Rooks/Bishops/Queens: Ray-walk with blocker attenuation (friendly: ÷1.5, enemy: ÷(2.0 + piece_weight × 0.3))
- Knights/Pawns/King: Direct projection to attack squares, no attenuation
- Output: `influence_grid[square][player]` — feeds swarm Layers 1-3

**Observer protocol:** Structured game JSON output for self-play, A/B testing, and training data extraction.
- Per-move: FEN4, eval vector, depth, component breakdown
- Per-game: result, player stats, move count
- Config-driven: engine version, search params, strategy profiles
- WebSocket-based Node.js service for real-time telemetry
- A/B runner for automated duels with statistical analysis

**Acceptance criteria:**

- [ ] Evaluator and Searcher traits defined and frozen
- [ ] Bootstrap evaluator: < 1μs per call, deterministic, no tactical terms
- [ ] PST verified correct for all four orientations
- [ ] Influence maps: < 1μs computation, blocker attenuation correct
- [ ] Observer captures structured game JSON with all required fields
- [ ] A/B runner executes configurable duels with result analysis

---

### Phase 3: OPPS + Swarm Co-development

**What you're building:** OPPS search and Swarm tactical resolution, developed together as a single integrated system. OPPS explores the game tree; Swarm assesses whether each leaf is tactically resolved. They cannot be built separately because swarm's stability signal determines OPPS's leaf boundaries.

**Why it's needed now:** This is the earliest phase where the engine can actually play moves.

**Depends on:** Phase 1 (Board, move generation), Phase 2 (Evaluator, influence maps, observer).

**Sub-phase 3a: Minimal OPPS with Stub Swarm**

Start with OPPS using the bootstrap evaluator directly, with swarm as a pass-through returning `(bootstrap_score, stability=1.0)`.

OPPS implementation:

- Paranoid search backbone: root player maximizes, all opponents minimize (from root player's perspective).
- Parameters: n1 (node budget), l1 (max paranoid depth), l2 (opponent pruning depth). Defaults: n1=50000, l1=12, l2=4.
- Alpha-beta pruning within paranoid framework. Valid because paranoid reduces to 2-player (Korf 1991).
- **Depth-4 rule enforced:** search only terminates at depths divisible by 4. Partial-round evaluations are meaningless in four-player.
- Move ordering: TT hints → killer moves → history heuristic → MVV-LVA.
- Transposition table with depth-4-aware replacement: only store entries at depths divisible by 4.

Verify at depth 4 with stub swarm: engine plays legal, non-suicidal moves.

**Sub-phase 3b: Swarm Layer Implementation**

Build the six swarm layers one at a time, A/B testing each layer's impact before adding the next. All layers live inside `compute_swarm()` or are called from it. Each layer feeds the next.

**Layer 1: Force Ratio.** For each square with pieces from multiple players: weight influence by piece value, compute attacker value vs defender value per player per square, flag squares as contested when 2+ players have significant force. This is a view on existing influence data — filtering and weighting.

**Layer 2: Pile-On Detection.** For each contested square: how many players have attackers (2 = duel, 3+ = pile-on)? Who has cheapest attackers (the lurker)? Who is most committed (the suckers)? Penalize the two most committed. Reward the lurker. Uses Lanchester square law: force effectiveness scales with influence².

**Layer 3: Chain Walk.** For contested squares flagged by Layers 1-2, resolve capture sequences statically (SUPER-SOMA adapted for 4 players):
1. Collect all attackers per player, sorted by piece value (ascending)
2. Turn order rotation (R→B→Y→G)
3. Each player: would I trade up or break even? If no: stand pat.
4. Continue until all pass or no attackers remain
5. Return material outcome per player: `chain_walk(state, square) -> [i16; 4]`

Branchless. O(n) per contested square. Most positions have 2-4 contested squares.

**Layer 4: Swarm-Delta.** For each piece identified as a chain walk participant: does removing it flip zone ownership? Drop any friendly piece from defended to undefended? Open a lane to your king? Not a full swarm recompute — check immediate neighbors using existing influence grid.

**Layer 5: Commitment Counting.** Across ALL contested squares: what fraction of total piece value is tied up in contested exchanges? How many fronts is this player committed on simultaneously? High ratio = overextended. Low ratio = flexible, dangerous.

**Layer 6: Chain Participation.** The synthesis layer. For each player on each contested square:
- **Initiate:** You win the exchange (L3), it doesn't cost you elsewhere (L4), you're not overextended (L5). Go.
- **Reinforce:** Marginal exchange, but one more piece tips it. Worth it if commitment ratio stays healthy.
- **Stay out:** You'd lose, or winning costs too much elsewhere, or you're already overextended.

**Swarm Aggregation:**

The six layers produce `(contribution, confidence)` pairs. Aggregation:

```
composite_score = Σ(contribution_i × confidence_i × weight_i) / Σ(confidence_i × weight_i)
stability = min(confidence_i for all i where weight_i > threshold)
```

Stability is conservative: minimum confidence across significant layers. If any layer is uncertain, the leaf is uncertain.

**Sub-phase 3c: Swarm-OPPS Integration**

Connect swarm's stability signal to OPPS's leaf determination:

```rust
fn evaluate_leaf(&mut self, state: &GameState, remaining_depth: u32) -> SwarmAssessment {
    let assessment = self.swarm.assess(state, &self.evaluator);
    if assessment.stability < self.stability_threshold && remaining_depth >= 4 {
        // Leaf is unstable — extend search by one full round
        return self.search_deeper(state, remaining_depth - 4);
    }
    assessment
}
```

Hard cap on extensions: no more than 2 additional rounds beyond base search depth. Prevents explosion.

Stability threshold calibrated by measuring:
1. How often extension changes leaf score by > 50cp (extensions that matter)
2. How often extension doesn't change score (wasted extensions)
3. Target: 15-30% extension rate, > 60% of extensions changing score by > 50cp

**Testing strategy (each layer A/B tested as added):**

1. Phase 0 baseline: stub swarm (pass-through) at depth 4, 8
2. Layer 1+2 duel: force ratio + pile-on vs bare eval, 30 games
3. Layer 3 duel: chain walk added vs without, 30 games
4. Layer 4+5 duel: swarm-delta + commitment vs without, 30 games
5. Layer 6 duel: full pipeline vs without participation layer, 30 games
6. Full pipeline vs stub swarm, 100 games

**Performance budget:** Bootstrap eval ~50-60μs release. Target with full swarm: < 120μs. Layers 1-2 ~5-10μs (views on existing data). Layer 3 ~10-20μs per contested square. Layers 4-5 ~5-10μs total. Layer 6 negligible.

**Known risks:**

1. **Swarm latency × OPPS leaf count.** Mitigation: progressive layer evaluation — Layers 1-2 mandatory and fast, Layers 3-6 only run if early layers haven't produced high confidence. Budget: < 2μs median at scale after optimization.

2. **Swarm-OPPS feedback loop.** Unstable → extend → still unstable → explosion. Mitigation: hard extension cap (2 additional rounds max).

3. **Nau's search pathology.** Uncorrelated sibling values → deeper search makes play worse. Swarm produces correlated sibling values because structural features change gradually between siblings. Verify: sibling correlation > 0.6 at depth 4 and 8.

4. **Baier & Kaisers' depth parity.** OPPS performs differently at even vs odd rounds. Depth-4 rule aligns with favorable parity. Verify: monotonic improvement at depth 4 → 8 → 12.

5. **Indirect tactics** (pins, forks, discoveries) not addressed by chain walk. Mitigated by keeping threat projection active in eval. Future: add as explicit NNUE features.

**Acceptance criteria:**

- [ ] OPPS returns legal best moves at depths 4, 8, 12
- [ ] Depth-4 rule enforced (rejects non-divisible-by-4)
- [ ] Alpha-beta pruning matches unpruned paranoid on small positions
- [ ] All six swarm layers produce plausible scores on test positions
- [ ] Extension rate within 15-30%, > 60% meaningful
- [ ] Sibling value correlation > 0.6 (anti-pathology)
- [ ] A/B test: OPPS+swarm > OPPS+bootstrap-only (100 games)
- [ ] TT with depth-4-aware replacement
- [ ] Swarm leaf evaluation median < 2μs (at scale, after optimization)
- [ ] Self-play games via observer produce valid structured JSON

---

### Phase 4: MCTS Strategic Layer

**What you're building:** Monte Carlo Tree Search as the strategic planning layer wrapping OPPS+Swarm, with Gumbel-Top-k selection, Opponent Move Abstraction, Progressive Widening, and a phase-separated hybrid controller.

**Why it's needed now:** OPPS is depth-first — good at tactics but not strategic breadth. MCTS compares fundamentally different plans by allocating simulation time to promising or uncertain candidates.

**Depends on:** Phase 3 (OPPS + Swarm). MCTS simulations are OPPS+Swarm evaluations.

**What's in scope:**

**Gumbel-Top-k root selection (from Freyja ADR-006).** At the root, use Gumbel noise-based selection with Sequential Halving instead of UCB1. UCB1 optimizes cumulative regret; the engine only plays one move, so simple regret matters. Gumbel-Top-k converges with 2 simulations vs 16+ for UCB1.

- Root policy: `pi(a) = softmax(ordering_score(a) / T)` where T=50
- Top-k: Retain 16 candidates (configurable `GUMBEL_K`)
- Sequential Halving: progressively eliminate candidates as simulations accumulate

**Opponent Move Abstraction (from Freyja ADR-018).** During MCTS simulations, opponent nodes pick ONE move via lightweight policy instead of expanding full tree:
- Priority: Checkmate > Capture (MVV-LVA) > Check > History > Random
- Impact: 3-4x deeper into root player's decision space

**Progressive Widening (from Freyja ADR-019).** At root-player MCTS nodes, limit children considered as `floor(k × visits^α)`, default k=2, α=0.5. Children sorted by prior descending so PW window exposes best moves first. Complements OMA: PW limits breadth at root-player nodes, OMA limits depth at opponent nodes.

**Phase-separated hybrid (from Freyja ADR-017).** Opening (ply < cutover): OPPS only, full time budget. Midgame (ply ≥ cutover): MCTS with OPPS simulations. Configurable `phase_cutover_ply`. Openings are structured — OPPS alone handles them well. MCTS adds value in chaotic midgame.

**Simulation.** Each simulation selects a leaf in the MCTS tree, then runs OPPS+Swarm to evaluate. OPPS depth for simulations is lower than final evaluation: depth 4 for simulations, depth 8-12 for final move decision.

**Backpropagation.** OPPS returns score from root player's perspective. Backpropagate through MCTS tree. At opponent nodes, the opponent model determines score interpretation.

**Opponent modeling.** Start with weighted mixture:
- 70% paranoid (opponents minimize your score)
- 20% max-own (opponents maximize their own score)
- 10% random (uniform over legal moves)

Tunable and can become position-dependent (more paranoid when leading, more max-own when opponents fight each other).

**Strategy profiles for training diversity (from Odin v1 ADR-017, Freyja ADR-014).** Two independent axes:
- Target selection: Vulture (lowest material), Predator (lowest king safety), Assassin (closest to elimination)
- Play style: Fortress (defensive), Territorial (space control)

These combine independently. Self-play uses all profiles for diverse training data.

**Known risks:**

1. **MCTS vs OPPS contradictory opponent models.** MCTS explores under mixed model, but OPPS evaluates under paranoid. Mitigation: OPPS evaluation is always paranoid (worst-case). MCTS opponent model only affects exploration, not evaluation.

2. **Simulation cost.** Each MCTS simulation is an OPPS search. Mitigation: shallow OPPS for simulations (depth 4), progressive deepening on most-visited nodes, TT shared across simulations.

3. **MCTS tree memory.** 4-player branching factor is large. Mitigation: tree pruning (remove conclusively inferior subtrees), clear aggressively.

**Acceptance criteria:**

- [ ] Gumbel-Top-k converges faster than UCB1 on test positions
- [ ] OMA produces 3-4x depth improvement in MCTS simulations
- [ ] Progressive Widening limits breadth without starving good moves
- [ ] Phase separation: OPPS-only in opening, MCTS in midgame
- [ ] MCTS+OPPS+Swarm beats OPPS+Swarm alone (200-game round-robin)
- [ ] Improvement concentrated in strategically complex (midgame) positions
- [ ] Time management: < 2s on forced positions, full budget on complex
- [ ] Strategy profiles produce diverse self-play games
- [ ] Memory usage < 512MB for MCTS tree under normal time controls
- [ ] TT shared correctly between simulations

---

### Phase 5: 1-Perspective NNUE

**What you're building:** A neural network evaluator replacing the bootstrap evaluator. Single accumulator, scalar output, relative encoding. Trained via generational self-play.

**Why it's needed now:** The bootstrap evaluator is hand-tuned and limited. The NNUE learns subtle positional patterns through self-play. It comes after MCTS because it needs a working engine to generate training data.

**Depends on:** Phase 4 (full engine capable of self-play), plus training pipeline.

**What's in scope:**

**Network architecture.** Input encoded from active player's perspective. All four players' pieces encoded relative to active player's orientation: self=0, next=1, across=2, prev=3. Feature transformer maps sparse board to dense vector. Two hidden layers (256 → 32 → 1) with clipped-ReLU. Scalar output in centipawn scale.

For positions needing other perspectives (MCTS backpropagation), rotate board 90°/180°/270° and do fresh forward pass. Acceptable because rotation only happens during MCTS backpropagation, not at every OPPS leaf. 1-perspective is correct for OPPS: paranoid only needs root player's score.

**Quantization:** Feature weights Q12 (×4096) to preserve small gradients. Hidden weights Q6 (×64). i16 little-endian, clamped [-32768, 32767].

**Incremental update.** When a move changes two squares, only two feature updates needed. Critical for OPPS leaf throughput.

**Training pipeline (Python + PyTorch):**

1. **Data generation.** Engine plays self-play games via observer. Games recorded as (position, search_score, game_outcome) tuples.
2. **Labeling.** Combined target: `target = λ × search_score + (1 - λ) × normalized_outcome`. Start with λ = 0.7. **Critical:** `game_outcome` (FFA points, 0-60 range) must be normalized to centipawn scale (-3000 to +3000) before blending with `search_score` (centipawns). See `4PC_RULES_REFERENCE.md` Section 9.4 for the full scoring pipeline. Raw FFA points blended with raw centipawns = corrupted training data.
3. **Training.** Generational loop:
   - Gen 1: Train from scratch on self-play data (LR=0.001)
   - Gen 2+: Warm-start from previous gen's checkpoint (LR=0.0001)
   - Accumulate ALL data across generations
   - EMA weight averaging (decay configurable)
   - Huber loss (robust to outliers) or MSE
   - Early stopping with patience
4. **Export.** Quantized weights to `.fnnue` binary. Verify nonzero weights.
5. **Iterate.** Engine with new NNUE plays better → generates better training data → next gen trains on richer data.

**Proof-of-value checkpoint:**

- NNUE score correlates with search score (R² > 0.7 on held-out positions)
- NNUE engine beats bootstrap engine in A/B testing (200 games)
- Incremental update < 500ns per position
- Rotation forward pass < 2μs
- Gen-2 NNUE beats gen-1 NNUE (self-improvement cycle works)

**Known risks:**

1. **Training signal poverty in gen-1.** Bootstrap evaluator is weak, so first-gen data is low quality. Mitigation: use search scores (ExIt — search is the "expert"), generate high volume (1M+ positions), use diverse strategy profiles.

2. **1-perspective rotation cost.** Mitigation: cache rotated evaluations, optimize rotation to lookup table, profile early.

3. **NNUE disrupting swarm stability.** Different score distribution than bootstrap may miscalibrate extensions. Mitigation: re-calibrate swarm stability threshold post-NNUE. Maintain 15-30% extension rate.

**Acceptance criteria:**

- [ ] NNUE implements Evaluator trait (drop-in replacement)
- [ ] Training pipeline converges (loss decreases over epochs)
- [ ] NNUE correlation with search score: R² > 0.7
- [ ] NNUE engine beats bootstrap engine in A/B testing
- [ ] Incremental update < 500ns
- [ ] Rotation < 2μs
- [ ] Gen-2 > gen-1 NNUE demonstrated
- [ ] Swarm stability re-calibrated post-NNUE
- [ ] No regression: nps > 80% of bootstrap nps

---

### Phase 6: UI and Full Integration

**What you're building:** The Tauri desktop application (TypeScript + React) and full integration of all engine components.

**Why it's needed now:** The engine is functional after Phase 4 and strong after Phase 5. Without a UI, only developers can use it. UI scaffolding can start at Phase 2; full integration requires Phase 4+.

**Depends on:** Phase 2 (API contract for rendering), Phase 4+ (engine for play).

**What's in scope:**

- **Board renderer.** 14×14 board, four players, color-coded. Dead zones visually distinct. Move highlighting, check indicators, smooth animation.
- **Game controls.** New game, position setup, step through moves, player assignment (human/engine per seat), time controls.
- **Engine analysis panel.** Real-time: search depth, nps, PV, eval per player, MCTS visit counts, swarm stability breakdown.
- **Tauri shell.** Engine as native sidecar. JSON protocol over stdin/stdout or local socket.

**Acceptance criteria:**

- [ ] Board renders correctly for all four players
- [ ] Human can play against engine
- [ ] Analysis panel shows real-time search info
- [ ] Observer telemetry displays without dropped messages
- [ ] UI responds within 16ms (60fps) during engine computation

---

## 4. Invariants

Properties that must hold throughout the entire project. Any change that violates an invariant is rejected.

### Board invariants

- `perft(1) = 20`, `perft(2) = 395`, `perft(3) = 7,800`, `perft(4) = 152,050`
- make/unmake round-trip preserves identical GameState including Zobrist hash
- No dead zone square ever appears in a move's source or destination
- Turn order is always R → B → Y → G, cycling
- Every move in the legal move list is actually legal
- DKW pieces are immovable and uncapturable

### Data structure invariants

- No `Vec<T>`, `Box<T>`, or heap allocation in Board, GameState, MoveUndo, or any hot-path type
- All hot-path types implement `Copy` or are stack-allocated with known size

### Search invariants

- OPPS only terminates at depths divisible by 4 (depth-4 rule)
- OPPS with no pruning matches full-width minimax on paranoid tree (verified on small positions)
- TT entries only created at depths divisible by 4
- Searcher::search() always returns a legal move

### Evaluation invariants

- Evaluator::evaluate() is a pure function: same GameState → same Score
- Bootstrap evaluator contains zero tactical terms
- Swarm assessment at every leaf: no leaf scored without running swarm pipeline (even as pass-through)

### Co-development invariants

- No stub evaluator in production search. Bootstrap is weak but real from Phase 2.
- No search without swarm at leaves. From Phase 3a onward, swarm always in loop.
- Searcher and Evaluator traits, once defined, are never modified. New capabilities via composition.

### NNUE invariants

- NNUE implements same Evaluator trait — drop-in replacement
- Board rotation is identity after 4 rotations
- Incremental update and full recomputation produce identical output

---

## 5. Risk Registry

| # | Risk | Severity | Phase | Mitigation | Detection |
|---|------|----------|-------|------------|-----------|
| R1 | Swarm latency × OPPS leaf count | High | 3 | Progressive layer evaluation; budget < 2μs; mandatory profiling | Leaf eval median > 2μs |
| R2 | Swarm-OPPS extension feedback loop | High | 3 | Hard extension cap (2 additional rounds max) | Node count > 10× expected |
| R3 | MCTS vs OPPS contradictory opponent models | Medium | 4 | OPPS always paranoid; MCTS model affects exploration only | Engine overvalues optimistic lines |
| R4 | 1-perspective rotation cost | Medium | 5 | Cache rotated evals; optimize to lookup table | Rotation > 15% of total eval time |
| R5 | Training signal poverty in gen-1 NNUE | High | 5 | Search scores as labels (ExIt); high volume; diverse profiles | Gen-1 NNUE fails to beat bootstrap |
| R6 | Nau search pathology | High | 3 | Swarm designed for correlated siblings; empirical measurement | Sibling correlation < 0.5 |
| R7 | Depth parity effects (Baier & Kaisers) | Medium | 3 | Depth-4 rule aligns with favorable parity; verify empirically | Non-monotonic improvement |
| R8 | NNUE disrupting swarm calibration | Medium | 5 | Re-calibrate stability threshold; measure extension rate | Extension rate outside 15-30% |
| R9 | Chain walk accuracy on complex exchanges | Medium | 3 | Pile-on detection catches lurker dynamics; A/B test | Chain walk disagreement with deep search > 20% |
| R10 | 14×14 board performance vs 8×8 | Low | 1 | Profile early; piece-list augmentation if needed | Move generation > 100ns per position |

---

## 6. Research References

**Baier, H. & Kaisers, M. (2020).** "Opponent-Pruning Paranoid Search." *FDG '20.*
Introduces OPPS with parameters n1, l1, l2. BRS+ is special case OPPS(1, ∞, 1). Reports depth parity issue and rand-Top-k finding. Tested on Chinese Checkers (3, 4, 6 players). Outperformed BRS+, Max^n, and Paranoid.

**Korf, R. E. (1991).** "Multi-Player Alpha-Beta Pruning." *Artificial Intelligence*, 48(1), 99-111.
Deep pruning in n-player requires reducing to 2-player. Justifies the paranoid backbone.

**Nau, D. S. (1982, 1983).** "An Investigation of the Causes of Pathology in Games" and "Pathology on Game Trees Revisited." *Artificial Intelligence.*
Deeper search can make play worse when sibling values are uncorrelated. Swarm is designed to produce correlated siblings.

**Schadd, M. P. D. & Winands, M. H. M. (2011).** "Best Reply Search for Multiplayer Games." *IEEE Trans. CI and AI in Games.*
BRS flattens opponent branching. Superseded by OPPS but informs opponent pruning design.

**Rollason, J. (2000).** "SUPER-SOMA." (SHOTEST, Shogi, world #3)
Branchless capture chain resolution replacing quiescence search. Direct precedent for chain walk.

**Schadd, M. P. D. & Winands, M. H. M. (2009).** EBQS for Stratego.
Static eval replacing qsearch. 56% win rate over depth-limited qsearch. Validates the approach when qsearch is expensive.

**Sturtevant, N. (2003).** "Last-Branch and Speculative Pruning for Max^n." *IJCAI-03.*
Pruning algorithms for constant-sum multiplayer games. Chinese Checkers depth 6: 1.2M → ~100k nodes.

**Zuckerman, I. et al. (2009).** "MP-Mix: Dynamic Strategy Switching." *IJCAI-09.*
Switches between Paranoid, Max^n, and directed offensive. Outperforms pure Paranoid.

**Silver, D. et al. (2018).** "A General Reinforcement Learning Algorithm." *Science.*
AlphaZero: search and evaluation co-develop through self-play.

**Anthony, T. et al. (2017).** "Thinking Fast and Slow with Deep Learning and Tree Search." *NeurIPS.*
Expert Iteration: search scores teach neural network to evaluate better than bootstrap.

**Nijssen, J. P. A. M. & Winands, M. H. M. (2010).** "Enhancements for Multi-Player MCTS."
Progressive history, MP-MCTS-Solver for multiplayer.

---

## 7. Glossary

| Term | Definition |
|------|-----------|
| OPPS | Opponent-Pruning Paranoid Search. Parameterized search generalizing paranoid with n1, l1, l2. |
| Paranoid | Search assumption: all opponents cooperate to minimize root player's score. |
| Swarm | Six-layer tactical assessment pipeline replacing quiescence search. |
| Stability | Swarm output in [0,1] indicating how resolved a position's tactical state is. |
| Bootstrap evaluator | Hand-crafted positional evaluator with no tactical terms. Swarm's base signal. |
| NNUE | Efficiently Updatable Neural Network. Replaces bootstrap evaluator after training. |
| Depth-4 rule | Search only terminates at depths divisible by 4 (one complete round). |
| OMA | Opponent Move Abstraction. Opponents pick 1 move via lightweight policy in MCTS. |
| PW | Progressive Widening. Limits MCTS children as floor(k × visits^α). |
| Gumbel-Top-k | Root selection policy using Gumbel noise + Sequential Halving. |
| DKW | Dead King Walking. Eliminated player's pieces become walls, king moves randomly. |
| Chain Walk | Branchless capture sequence resolution (SUPER-SOMA adapted for 4 players). |
| ExIt | Expert Iteration. Search provides expert labels for training the neural network. |
| GPI | Generalized Policy Iteration. Alternating improvement of policy (search) and value (eval). |
| MVV-LVA | Most Valuable Victim – Least Valuable Attacker. Capture ordering heuristic. |

---

## 8. Phase Dependency Graph

```
Phase 1: Board + Rules + MoveGen + DKW
    │
    ▼
Phase 2: Traits + Bootstrap Eval + Influence Maps + Observer
    │
    ▼
Phase 3: OPPS + Swarm (co-developed)
    │    3a: OPPS with stub swarm
    │    3b: Swarm layers (1-6), each A/B tested
    │    3c: Swarm-OPPS integration (stability threshold)
    │
    ▼
Phase 4: MCTS Strategic Layer
    │    Gumbel-Top-k + OMA + PW + Phase Separation
    │    Strategy profiles for training diversity
    │
    ▼
Phase 5: 1-Perspective NNUE + Training Pipeline
    │    Generational self-play → warm-start → export
    │
    ▼
Phase 6: UI + Full Integration
           (scaffolding starts at Phase 2,
            full integration at Phase 4+)
```

Each arrow represents a hard dependency. Phase 6's UI scaffolding is the exception — it can proceed in parallel once trait definitions from Phase 2 are established.

---

*This document is the governing plan for Project Valhalla. Changes to architectural decisions, invariants, or trait interfaces require explicit rationale documented here before implementation.*
