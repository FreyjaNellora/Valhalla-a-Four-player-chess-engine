# Valhalla

**A novel four-player chess engine built from first principles.**

Valhalla plays 14x14 free-for-all four-player chess. It does not adapt two-player techniques to four players — it is designed from scratch around the structural reality that four-player search is a fundamentally different problem.

---

## Architecture

```
MCTS Strategic Layer
  Gumbel-Top-k selection + Sequential Halving
  Opponent Move Abstraction (3-4x deeper simulations)
  Progressive Widening at root-player nodes
  Phase-separated hybrid (OPPS opening, MCTS midgame)
    |
OPPS Tactical Search
  Paranoid backbone + parameterized opponent pruning (Baier & Kaisers 2020)
  Alpha-beta pruning via paranoid reduction to 2-player
  Depth-4 rule: search terminates only at full rounds (d=4,8,12...)
    |
Swarm Tactical Resolution (replaces quiescence search)
  L1: Force Ratio         — contested squares, balance of power
  L2: Pile-On Detection   — lurkers vs committed (Lanchester square law)
  L3: Chain Walk          — branchless capture resolution (SUPER-SOMA)
  L4: Swarm-Delta         — cost of engaging elsewhere
  L5: Commitment Counting — overextension across fronts
  L6: Chain Participation — initiate, reinforce, or stay out
    |
1-Perspective NNUE
  Relative player encoding (self/next/across/prev)
  Generational self-play training with warm-start
  Quantized fixed-point inference (Q12 features, Q6 hidden)
```

## Why This Exists

In two-player chess, deep pruning makes engines strong. Alpha-beta search with a decent eval function has been the dominant approach for decades.

In four-player chess, that doesn't work:

- **Deep pruning is provably limited** for n>2 players (Korf 1991)
- **Quiescence search breaks down** when three opponents move between your turns
- **Search pathology is a real threat** — deeper search can make play *worse* (Nau 1982-83)

Valhalla addresses each of these:

- **OPPS** reduces to 2-player via paranoid assumption, enabling alpha-beta where Max^n cannot
- **Swarm** replaces quiescence with a six-layer tactical pipeline that resolves captures, detects pile-ons, and assesses commitment risk without tree expansion
- **Correlated sibling values** from swarm's structural features prevent search pathology

## Heritage

Valhalla is the third engine in a lineage:

- **Odin v1** (complete, 19 stages) — BRS/Paranoid hybrid. Proved the engineering patterns: fixed-size data, attack query API, player-aware TT, ArrayVec movegen.
- **Freyja** (in progress, 17 stages) — Pure Max^n with NNUE-guided beam search. Proved that Max^n is impractical for deep search in 4PC. Contributed Gumbel-Top-k MCTS, OMA, Progressive Widening, phase-separated hybrid, DKW rules, ray-attenuated influence maps, and the generational NNUE training methodology.
- **Valhalla** (this project) — Takes OPPS from the research literature, Swarm from novel design, and proven components from both predecessors.

When both Freyja and Valhalla are complete, they compete head-to-head: Max^n vs OPPS, two philosophies of multiplayer game tree search.

## Tech Stack

| Component | Technology |
|-----------|-----------|
| Engine | Rust |
| UI | TypeScript + React + Tauri |
| Training | Python + PyTorch |
| Observer | Node.js |

## Research Foundation

Valhalla's design is informed by published research where it exists, and explicitly novel where it doesn't:

| Component | Research Status |
|-----------|---------------|
| OPPS search | Published — Baier & Kaisers 2020 |
| Paranoid reduction | Established — Korf 1991, Sturtevant 2000 |
| MCTS for multiplayer | Published — Nijssen 2013, Sturtevant 2008 |
| Swarm leaf evaluation | **Novel** — no published precedent |
| Chain walk (multiplayer SEE) | **Novel** — extends SUPER-SOMA (Rollason 2000) |
| Commitment risk in search | **Novel** — no formal framework exists |
| Gumbel-Top-k for game engines | Applied from Danihelka et al. 2022 |
| Expert Iteration training | Published — Anthony et al. 2017, Silver et al. 2018 |

Full research library with implementation notes: [`masterplan/research/`](masterplan/research/)

## Project Status

**Pre-Phase 0** — Architecture designed, operational framework complete, ready to build.

| Phase | Name | Status |
|-------|------|--------|
| 1 | Board + Rules + MoveGen + DKW | Not started |
| 2 | Bootstrap Eval + Influence Maps + Observer | Not started |
| 3 | OPPS + Swarm Co-development | Not started |
| 4 | MCTS Strategic Layer | Not started |
| 5 | NNUE + Training Pipeline | Not started |
| 6 | UI + Full Integration | Not started |

## Key Design Decisions

- **No quiescence search.** Swarm replaces it entirely. Qsearch is a 2-player technique that doesn't scale to 4 players.
- **No CNS (Conspiracy Number Search).** No published extension to multiplayer exists. The board is too volatile — three opponents can restructure the position between your moves.
- **1-perspective NNUE, not 4.** OPPS only needs root player's score (paranoid min). Cheaper than maintaining 4 accumulators.
- **Depth-4 rule.** Search only terminates at depths divisible by 4 (one full round). Partial rounds create evaluation bias.
- **Fixed-size data structures.** No `Vec<T>` on the hot path. O(1) clone, zero allocation during search.

## License

MIT

## Author

Built by Nell with AI collaboration (Claude, Anthropic).
