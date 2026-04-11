# Fog of Intention — Opponent Inference in Multiplayer Games

**Status: NO formal framework for multiplayer board games. Adjacent work exists in poker, Diplomacy, and RTS.**

## The Core Problem

In 4-player chess, there is no literal fog of war — all pieces are visible. But there IS **fog of intention**: you know what 3 opponents CAN do but not what they WILL do. This creates uncertainty that pure Max^n and Paranoid search handle crudely:

- **Max^n**: assumes all opponents play optimally for themselves (often wrong — opponents make mistakes, have grudges, or target specific players)
- **Paranoid**: assumes all opponents coordinate against you (worst-case, often too pessimistic)
- **OPPS**: paranoid within the search tree, but doesn't update beliefs based on observed behavior

The missing capability: **Bayesian updating of opponent intent based on observed move history.**

## What Does Exist

### Poker (Most Mature Framework)

The gold standard for decision-making under hidden information with multiple opponents:
- **Range estimation** — narrowing opponent's likely holdings based on their betting patterns
- **Bayesian updating** — each opponent action updates belief distribution
- **Pot commitment** — sunk cost changes future decisions (maps to material commitment)
- **Multi-way pot dynamics** — with 3+ players, you must model interactions between opponents, not just you-vs-them
- **GTO vs exploitative play** — baseline strategy vs adapting to opponent tendencies

The poker framework is mathematically rigorous and directly applicable: replace "card range" with "likely move set" and "betting action" with "piece movement pattern."

### Diplomacy / CICERO (Meta AI, 2022)

- **Intent prediction** — predicting what each opponent will actually do, not assuming optimal or worst-case
- **piKL planning** — models opponent responses to your commitments
- **Commitment detection** — tracking whether opponents follow through on stated/implied intentions
- 7-player game with alliances, betrayal, coordination — more socially complex than 4PC but similar uncertainty structure

Source: Bakhtin et al. "Human-level play in the game of Diplomacy." Science, 2022.
URL: https://www.science.org/doi/10.1126/science.ade9097

### RTS Games / StarCraft (Literal Fog of War)

- **Scouting value** — making sub-optimal moves to gain information
- **Build order prediction** — Bayesian inference of opponent strategy from partial observations
- **Threat estimation under uncertainty** — estimating army composition from limited scouting data
- **AlphaStar** used opponent modeling in self-play but didn't publish the inference framework in detail

### Go (Tenuki and Thickness)

- **Tenuki** — playing elsewhere instead of responding locally. The decision to redirect attention.
- **Thickness** — influence that doesn't require further investment. Low attention cost.
- **Sente/Gote** — who controls the flow of attention. Sente = forcing opponent to respond. Gote = opponent ignores you.
- These concepts are deeply integrated into Go evaluation but never formalized as "fog of intention" math.

### Chinese Checkers / Multi-Player Abstract Games

- Path commitment — once pieces move in one direction, reversing is costly
- Coalition detection — are two opponents coordinating against you?
- No published opponent modeling frameworks for abstract multiplayer board games

## What Would Need to Be Invented

A fog-of-intention framework for 4PC would need:

1. **Threat probability distribution** — "given this board state, there's a 60% chance Blue attacks me and 40% chance Blue attacks Yellow." Not binary (paranoid vs max-own) but a continuous spectrum.

2. **Bayesian updating from observed moves** — "Blue moved their knight toward my king 3 turns in a row. P(Blue targets me) increases from 0.4 to 0.8." Each observed move narrows the probability distribution.

3. **Information value of moves** — "Is it worth making a slightly sub-optimal move to probe Blue's intentions?" A scouting concept borrowed from RTS/poker.

4. **Multi-opponent interaction modeling** — "If Blue attacks me, does that make Yellow more or less likely to also attack me?" Pile-on dynamics vs opportunistic flanking.

5. **Decay of certainty** — opponent intention estimates should decay over time if no confirming evidence arrives. A player who was attacking you 10 moves ago may have switched targets.

## How This Maps to Valhalla's Architecture

- **Phase 4 (Strategy & Opponent Modeling)** is where this lands
- Valhalla already plans a 70% paranoid / 20% max-own / 10% random opponent model mixture
- Fog-of-intention would make that mixture **position-dependent and history-dependent** instead of fixed weights
- The MCTS tree naturally explores different opponent responses — Bayesian updating would bias exploration toward more likely opponent actions
- Connects to MP-Mix (Zuckerman 2009): dynamic strategy switching based on opponent interaction level

## Cross-References

- `commitment-risk-novel.md` — Commitment risk IS the opportunity cost of attention (same concept, different name)
- `cicero-meta-diplomacy-2022.md` — CICERO's intent prediction is the closest existing implementation
- `MP-Mix-zuckerman-2009.md` — Dynamic strategy switching informs position-dependent opponent modeling
- `lanchester-attrition-models.md` — Force allocation across multiple adversaries
