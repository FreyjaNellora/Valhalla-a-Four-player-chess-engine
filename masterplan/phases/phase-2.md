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
| **Status** | not-started |
| **Last Session** | -- |
| **Blocking Issues** | Phase 1 not complete |

## Acceptance Checklist

- [ ] Evaluator and Searcher traits defined and frozen
- [ ] Bootstrap evaluator: < 1us per call, deterministic, no tactical terms
- [ ] PST verified correct for all four orientations
- [ ] Influence maps: < 1us computation, blocker attenuation correct
- [ ] Observer captures structured game JSON with all required fields
- [ ] A/B runner executes configurable duels with result analysis

## Active Watch Items

- **If Evaluator trait signature needs changing after freeze:** This is a change order. Stop, document why, get approval.
- **If influence maps exceed 1us:** Profile blocker attenuation. Consider lazy computation (only recompute changed rays).
- **If bootstrap eval produces non-monotonic material scores:** Check for accidental lead penalty or tactical terms creeping in.

## Rework Log

| Date | What Changed | Why | Impact |
|------|-------------|-----|--------|
| | | | |

## Downstream Notes

Phase 3 needs:
- `Evaluator` trait: `fn evaluate(&self, state: &GameState) -> Score`
- `Searcher` trait: `fn search(&mut self, state: &GameState, depth: u32) -> SearchResult`
- Bootstrap evaluator implementing `Evaluator` (material + PST + king safety + pawn structure)
- `influence_grid[square][player]` data structure, recomputable per position
- Observer protocol: game JSON format, A/B runner API
- Protocol LogFile for diagnostic sessions
