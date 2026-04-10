# Phase 5: 1-Perspective NNUE + Training Pipeline

## Commander's Intent

Replace the bootstrap evaluator with a learned one. Train via generational self-play: the engine plays games, labels positions, trains a network, exports weights, and repeats. The NNUE must beat bootstrap in A/B testing, and gen-2 must beat gen-1 to prove the self-improvement cycle works. Keep the 1-perspective design — it is correct for OPPS's paranoid backbone.

## Reading List

1. `masterplan/MASTERPLAN.md` Section 3 — Phase 5 specification
2. `masterplan/MASTERPLAN.md` Section 5 — Risk Registry (R4, R5, R8)
3. `masterplan/MASTERPLAN.md` Section 4 — Invariants (NNUE)
4. `masterplan/MASTERPLAN.md` Section 6 — Research references (Silver 2018, Anthony 2017 — ExIt/GPI)
5. `masterplan/DECISIONS.md` — ADR-003 (Evaluator Trait), ADR-015 (SIMD-Ready NNUE)
6. `masterplan/phases/phase-4.md` — Downstream notes (self-play, observer, training tuples)
7. `masterplan/SYSTEM_PROFILE.local.md` — GPU, RAM constraints for training

## Write Scope

- `valhalla-engine/src/nnue/` — all files (architecture, forward pass, incremental update, quantization, SIMD)
- `valhalla-nnue/` — all files (Python training pipeline: data loading, model, training loop, export)
- `valhalla-engine/src/eval/` — NNUE evaluator implementing Evaluator trait (new file, does not modify bootstrap)
- Weight files: `*.fnnue` binary format
- Observer configs for NNUE vs bootstrap duels
- Tests for all of the above

## Current State

| Field | Value |
|-------|-------|
| **Status** | not-started |
| **Last Session** | -- |
| **Blocking Issues** | Phases 1-4 not complete |

## Acceptance Checklist

- [ ] NNUE implements Evaluator trait (drop-in replacement)
- [ ] Training pipeline converges (loss decreases over epochs)
- [ ] NNUE correlation with search score: R-squared > 0.7
- [ ] NNUE engine beats bootstrap engine in A/B testing
- [ ] Incremental update < 500ns
- [ ] Rotation < 2us
- [ ] Gen-2 > gen-1 NNUE demonstrated
- [ ] Swarm stability re-calibrated post-NNUE
- [ ] No regression: nps > 80% of bootstrap nps

## Active Watch Items

- **If rotation cost exceeds 15% of eval time (R4):** Cache rotated evaluations. Optimize rotation to lookup table.
- **If gen-1 NNUE fails to beat bootstrap (R5):** Use search scores as labels (ExIt). Generate higher volume (1M+ positions). Use all strategy profiles.
- **If NNUE disrupts swarm stability (R8):** Re-calibrate stability threshold. Target 15-30% extension rate. Different score distribution is expected — the threshold needs tuning, not the NNUE.
- **If training OOMs on local GPU:** Reduce batch size. Freyja experience: 4-6 game batches for self-play, small training batches.

## Rework Log

| Date | What Changed | Why | Impact |
|------|-------------|-----|--------|
| | | | |

## Downstream Notes

Phase 6 needs:
- NNUE `.fnnue` weight file loadable at engine startup
- Engine selectable between bootstrap and NNUE evaluator via config/protocol
- Eval breakdown available for UI analysis panel (component scores)
- Stable, tested engine ready for end-user play
