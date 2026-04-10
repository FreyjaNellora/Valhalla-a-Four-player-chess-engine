# Phase 4: MCTS Strategic Layer

## Commander's Intent

Wrap OPPS+Swarm in an MCTS layer that handles strategic breadth. Use Gumbel-Top-k for efficient root selection, OMA to deepen simulations, Progressive Widening to control breadth, and a phase-separated controller that knows when MCTS adds value (chaotic midgame) vs when OPPS alone suffices (structured openings). By the end, MCTS+OPPS+Swarm must beat OPPS+Swarm alone in a 200-game round-robin.

## Reading List

1. `masterplan/MASTERPLAN.md` Section 3 — Phase 4 specification
2. `masterplan/MASTERPLAN.md` Section 5 — Risk Registry (R3)
3. `masterplan/MASTERPLAN.md` Section 6 — Research references (Silver 2018, Anthony 2017, Nijssen 2010)
4. `masterplan/research/OPPS-baier-kaisers-2020.md` — OPPS as simulation backbone
5. `masterplan/research/MCTS-multiplayer-nijssen-2013.md` — Progressive History, MP-MCTS-Solver
6. `masterplan/research/MP-Mix-zuckerman-2009.md` — Dynamic strategy switching, opponent modeling
7. `masterplan/DECISIONS.md` — ADR-011 (Gumbel over UCB1), ADR-012 (Progressive History BRS->MCTS), ADR-017 (Strategy Profiles)
8. `masterplan/phases/phase-3.md` — Downstream notes (OPPS as simulation engine, TT sharing, history table)
9. `masterplan/SYSTEM_PROFILE.local.md` — Memory constraints for MCTS tree

## Write Scope

- `valhalla-engine/src/mcts/` — all files (tree, selection, backpropagation, Gumbel, OMA, PW)
- `valhalla-engine/src/hybrid/` — all files (phase-separated controller, time management)
- `valhalla-engine/src/strategy/` — all files (opponent modeling, strategy profiles)
- Observer configs for MCTS vs OPPS duels and strategy profile diversity tests
- Tests for all of the above

## Current State

| Field | Value |
|-------|-------|
| **Status** | not-started |
| **Last Session** | -- |
| **Blocking Issues** | Phases 1-3 not complete |

## Acceptance Checklist

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

## Active Watch Items

- **If MCTS+OPPS contradictory opponent models cause issues (R3):** OPPS evaluation is always paranoid. MCTS model only affects exploration. If engine overvalues optimistic lines, tighten MCTS paranoid weight.
- **If simulation cost is too high:** Reduce OPPS simulation depth to 4. Enable TT sharing. Profile node reuse.
- **If MCTS tree exceeds 512MB:** Prune conclusively inferior subtrees more aggressively. Clear tree between moves.
- **If strategy profiles produce homogeneous games:** Increase profile diversity. Check that blend weights actually differentiate play.

## Rework Log

| Date | What Changed | Why | Impact |
|------|-------------|-----|--------|
| | | | |

## Downstream Notes

Phase 5 needs:
- Full engine capable of self-play at configurable depth and time budgets
- Observer pipeline producing `(position, search_score, game_outcome)` training tuples
- Strategy profiles generating diverse games for training data variety
- Stable, reproducible self-play with deterministic seeding option
- Engine API that accepts an `Evaluator` as a pluggable component (for NNUE swap)
