# Phase 3: OPPS + Swarm Co-development

## Commander's Intent

Build the search and tactical resolution system as one integrated unit. OPPS searches the tree; swarm tells OPPS whether a leaf is resolved. They cannot be built separately. This is the phase where the engine starts playing moves. By the end, OPPS+Swarm must demonstrably beat OPPS+bootstrap-only in A/B testing.

## Reading List

1. `masterplan/MASTERPLAN.md` Section 3 — Phase 3 specification (sub-phases 3a, 3b, 3c)
2. `masterplan/MASTERPLAN.md` Section 5 — Risk Registry (R1, R2, R6, R7, R9)
3. `masterplan/MASTERPLAN.md` Section 4 — Invariants (search, co-development)
4. `masterplan/research/OPPS-baier-kaisers-2020.md` — OPPS algorithm, parameters, depth parity
5. `masterplan/research/SUPER-SOMA-rollason-2000.md` — Branchless capture resolution (chain walk precedent)
6. `masterplan/research/EBQS-schadd-winands-2009.md` — Static eval replacing qsearch
7. `masterplan/DECISIONS.md` — ADR-002 (BRS/Paranoid, now superseded by OPPS), ADR-009 (Eval/Search Separation)
8. `masterplan/phases/phase-1.md` — Downstream notes
9. `masterplan/phases/phase-2.md` — Downstream notes (traits, influence maps, observer)
10. `masterplan/SYSTEM_PROFILE.local.md` — Hardware constraints

## Write Scope

- `valhalla-engine/src/search/` — all files (OPPS implementation, TT, move ordering, killer/history)
- `valhalla-engine/src/swarm/` — all files (six layers, aggregation, stability signal)
- `valhalla-engine/src/tt/` — all files (transposition table, depth-4-aware replacement)
- Observer configs and test scripts for A/B duels
- Tests for all of the above

## Current State

| Field | Value |
|-------|-------|
| **Status** | not-started |
| **Last Session** | -- |
| **Blocking Issues** | Phases 1-2 not complete |

## Acceptance Checklist

- [ ] OPPS returns legal best moves at depths 4, 8, 12
- [ ] Depth-4 rule enforced (rejects non-divisible-by-4)
- [ ] Alpha-beta pruning matches unpruned paranoid on small positions
- [ ] All six swarm layers produce plausible scores on test positions
- [ ] Extension rate within 15-30%, > 60% meaningful
- [ ] Sibling value correlation > 0.6 (anti-pathology)
- [ ] A/B test: OPPS+swarm > OPPS+bootstrap-only (100 games)
- [ ] TT with depth-4-aware replacement
- [ ] Swarm leaf evaluation median < 2us (at scale, after optimization)
- [ ] Self-play games via observer produce valid structured JSON

## Active Watch Items

- **If swarm latency exceeds 2us median (R1):** Implement progressive layer evaluation — Layers 1-2 mandatory, Layers 3-6 only if early layers have low confidence.
- **If extension loop explodes (R2):** Hard cap is 2 additional rounds. If hitting cap frequently, stability threshold is too low.
- **If sibling correlation < 0.5 (R6):** Nau pathology risk. Investigate which swarm layers are producing uncorrelated values.
- **If depth 4 to 8 shows non-monotonic improvement (R7):** Depth parity issue. Verify OPPS parameters.
- **If chain walk disagrees with deep search > 20% (R9):** Chain walk is too simple for the exchange. Consider adding a pass for indirect tactics.

## Rework Log

| Date | What Changed | Why | Impact |
|------|-------------|-----|--------|
| | | | |

## Downstream Notes

Phase 4 needs:
- `Searcher` implementation (OPPS) that Phase 4's MCTS can call as simulation engine
- Swarm's `SwarmAssessment` struct with `score` and `stability`
- TT that can be shared across MCTS simulations
- History table data for progressive history warm-start (ADR-012)
- Move ordering scores for MCTS prior policy initialization
- Observer integration: self-play producing valid game JSON
