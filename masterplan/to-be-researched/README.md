# To Be Researched

Research TODOs tagged by phase. The agent working a given phase should investigate these topics before or during implementation. Each entry includes what's known so far and where to start.

Items marked **BLOCKING** must be resolved before the phase can proceed. Items marked **ENRICHING** improve quality but don't block progress.

---

## Phase 3 — Swarm Tactical Resolution

### Opportunity Cost of Attention (ENRICHING)
**AKA:** Commitment risk, fog of intention, attention distribution
**What we know:** Commitment risk (Layer 5) measures HOW MUCH you've committed. Swarm-delta (Layer 4) measures WHAT YOU LOSE by committing. But neither measures the GLOBAL attention distribution — are you neglecting entire board sectors?
**Research needed:**
- How do poker engines model pot commitment across multiple opponents?
- Can we divide the 14x14 board into sectors (one per opponent's home zone) and penalize extreme influence concentration?
- Should the penalty scale with opponent strength in the neglected sector?
**Start with:** `research/fog-of-intention-opponent-inference.md`, `research/commitment-risk-novel.md`, `research/lanchester-attrition-models.md`

### Chain Walk Accuracy vs Speed Tradeoff (ENRICHING)
**What we know:** Chain walk (Layer 3) resolves captures without search. SUPER-SOMA does this for 2-player. Extending to 4-player means any of 3 opponents can intervene in a capture chain.
**Research needed:**
- How deep should chain walks go before cutting off?
- When 3+ players are involved in a capture chain, which orderings matter most?
- Can we use Lanchester force ratios to prune unlikely chain continuations?
**Start with:** `research/SUPER-SOMA-rollason-2000.md`, `research/multiplayer-see-novel.md`, `research/EBQS-schadd-winands-2009.md`

---

## Phase 4 — Strategy & Opponent Modeling

### Bayesian Opponent Intent Inference (ENRICHING)
**What we know:** Valhalla plans a fixed 70/20/10 paranoid/max-own/random opponent model. This could be made position-dependent and history-dependent using Bayesian updating from observed moves.
**Research needed:**
- How does CICERO's intent prediction actually work at the implementation level?
- Can we maintain a lightweight belief distribution per opponent that updates each move?
- What's the computational cost of Bayesian updating in the MCTS loop?
- How does poker's range narrowing map to move prediction?
**Start with:** `research/fog-of-intention-opponent-inference.md`, `research/cicero-meta-diplomacy-2022.md`, `research/MP-Mix-zuckerman-2009.md`

### Strategy Profile Detection (ENRICHING)
**What we know:** We plan vulture/predator/assassin profiles for self-play diversity. But can we also DETECT which profile an opponent is playing?
**Research needed:**
- What observable features distinguish aggressive vs defensive vs opportunistic play?
- How quickly can we classify an opponent's style (how many moves needed)?
- Should detection feed back into the opponent model mixture weights?
**Start with:** `research/MP-Mix-zuckerman-2009.md`, `research/cicero-meta-diplomacy-2022.md`

---

## Phase 5 — NNUE Training Pipeline

### Training Target Normalization (BLOCKING)
**What we know:** `target = lambda * search_score + (1 - lambda) * normalized_outcome`. Game outcomes (FFA: 0-60 points) must be normalized to centipawn scale (-3000 to +3000) before blending. The normalization method matters.
**Research needed:**
- Linear scaling? Sigmoid? Z-score?
- What does Stockfish NNUE training use for outcome blending?
- Should the normalization adapt as the eval scale shifts across generations?
**Start with:** `research/alphazero-silver-2018.md`, `research/expert-iteration-anthony-2017.md`

### Self-Play Diversity Without Strategy Profiles (ENRICHING)
**What we know:** Noise injection (move_noise) provides some diversity. Strategy profiles add more. But are there other approaches?
**Research needed:**
- AlphaZero uses temperature-based move selection — how effective is this for 4-player?
- Population-based training (PBT) for exploring eval weight space?
- Adversarial self-play (deliberately training against weaknesses)?
**Start with:** `research/alphazero-silver-2018.md`, `research/expert-iteration-anthony-2017.md`

---

## Phase 6 — UI & Integration

No research TODOs identified yet. Phase 6 is primarily engineering (Tauri/React).
