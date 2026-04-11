# Phase 2: Bootstrap Evaluator, Influence Maps, and Observer

## Commander's Intent

Define the frozen trait interfaces that every future phase builds on. Build a real (not stub) evaluator that is purely positional — no tactical terms, because swarm owns tactics. Build the influence map infrastructure that swarm will consume. Stand up the observer pipeline so we can measure everything from Phase 3 onward.

## Reading List

1. `masterplan/MASTERPLAN.md` Section 3 — Phase 2 specification
2. `masterplan/MASTERPLAN.md` Section 2 — Architecture overview (frozen traits, influence maps)
3. `masterplan/MASTERPLAN.md` Section 4 — Invariants (evaluation, co-development)
4. `masterplan/DECISIONS.md` — ADR-003 (Evaluator Trait), ADR-007 (Protocol LogFile), ADR-008 (Observer Pipeline), ADR-009 (Eval/Search Separation), ADR-010 (No Lead Penalty)
5. `masterplan/phases/phase-1.md` — Downstream notes (what Phase 1 provides)
6. `masterplan/SYSTEM_PROFILE.local.md` — Hardware constraints for performance budgets

## Write Scope

- `valhalla-engine/src/eval/` — all files (bootstrap evaluator, PST, trait definition)
- `valhalla-engine/src/search/` — trait definition only (`Searcher` trait)
- `valhalla-engine/src/influence/` — all files (ray-attenuated influence maps)
- `valhalla-engine/src/protocol/` — all files (engine protocol, LogFile)
- `observer/` — all files (Node.js observer pipeline)
- Tests for all of the above

## Current State

| Field | Value |
|-------|-------|
| **Status** | implementation complete — pending user testing |
| **Last Session** | 2026-04-11 |
| **Blocking Issues** | None. Observer deferred to Phase 3. |

## Acceptance Checklist

- [x] Evaluator and Searcher traits defined and frozen
- [x] Bootstrap evaluator: < 1us per call, deterministic, no tactical terms
- [x] Bootstrap evaluator exposes `evaluate_breakdown()` with labeled component scores
- [x] PST verified correct for all four orientations
- [x] Influence maps: < 1us computation, compounding blocker attenuation correct
- [x] Influence maps skip invalid corner squares (no compute-then-zero)
- [x] CLAUDE.md rule 9 updated to `depth: u32`
- [ ] Observer captures structured game JSON with all required fields — **deferred to Phase 3**
- [ ] A/B runner executes configurable duels with result analysis — **deferred to Phase 3**

## Active Watch Items

- **If Evaluator trait signature needs changing after freeze:** This is a change order. Stop, document why, get approval.
- **If influence maps exceed 1us:** Profile blocker attenuation. Consider lazy computation (only recompute changed rays).
- **If bootstrap eval produces non-monotonic material scores:** Check for accidental lead penalty or tactical terms creeping in.

## Rework Log

| Date | What Changed | Why | Impact |
|------|-------------|-----|--------|
| 2026-04-11 | Observer deferred to Phase 3 | Nothing to observe yet — engine can't play moves until Phase 3 | Phase 3 scope expanded |
| 2026-04-11 | B=450cp confirmed | User specified B = R - 50cp | constants.rs already had this from prior session |

## Downstream Notes

Phase 3 needs:
- `Evaluator` trait: `fn evaluate(&self, state: &GameState) -> Score`
- `Searcher` trait: `fn search(&mut self, state: &GameState, depth: u32) -> SearchResult`
- `BootstrapEvaluator` implementing `Evaluator` + `evaluate_breakdown() -> EvalBreakdown`
- `InfluenceMap::compute(&GameState)` — `grid[sq][player]` with compounding blocker gradient
- `ProtocolLog` for diagnostic logging
- Observer protocol (deferred from Phase 2): game JSON, A/B runner, WebSocket server
