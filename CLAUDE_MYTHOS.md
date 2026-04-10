# Project Valhalla -- Agent Orientation (Mythos)

> **You are reading the Mythos variant.** All hard constraints, correctness rules, approval gates, and domain knowledge are identical to `CLAUDE.md`. Procedural scaffolding and compensating guardrails have been removed. **Principle: Less procedure. Same knowledge. Same boundaries.**

A four-player chess engine: OPPS + Swarm + MCTS + NNUE.
Clean rebuild incorporating lessons from Odin v1 (Stages 0-19) and Freyja (Stages 0-17).

---

## Orientation

Read `masterplan/STATUS.md` and `masterplan/HANDOFF.md` to understand current project state. Explore further as needed — `masterplan/MASTERPLAN.md` for stage specs, `masterplan/DECISIONS.md` for architectural rationale, `masterplan/AGENT_CONDUCT_MYTHOS.md` for behavior rules.

Knowledge vault at `masterplan/`:
- Big picture: `_index/MOC-Project-Valhalla.md`
- Active issues: `_index/MOC-Active-Issues.md`
- Session history: `_index/MOC-Sessions.md`
- All wikilink targets: `_index/Wikilink-Registry.md`

---

## What Goes Where

| Content | Where | Rule |
|---|---|---|
| Stage specs, acceptance criteria | `masterplan/MASTERPLAN.md` | Authoritative. Never duplicate elsewhere. |
| ADRs, architectural decisions | `masterplan/DECISIONS.md` | Why key choices were made. |
| Agent behavior rules | `masterplan/AGENT_CONDUCT_MYTHOS.md` | HOW agents work. |
| Audit logs, downstream logs | `masterplan/` | Formal records per stage. |
| Project state, session handoff | `masterplan/STATUS.md` + `HANDOFF.md` | Update at session end. |
| Implementation knowledge | `masterplan/components/` | How things work at code level. |
| Component relationships | `masterplan/connections/` | How things connect. |
| Session history | `masterplan/sessions/` | Preserved history. |
| Bugs, workarounds | `masterplan/issues/` | Runtime problems. |
| Implementation patterns | `masterplan/patterns/` | Reusable approaches. |

---

## Key Differences from v1 (Pitfall Table)

**From Stages 0-9:**
1. **No Huginn.** `tracing` crate from Stage 0. Protocol LogFile from Stage 4. Observer from Stage 6.
2. **Fixed-size data from Stage 1.** No Vec in piece_lists or position_history. Clone is O(1).
3. **TT player-aware from Stage 1.** Zobrist includes root_player keys.
4. **Behavioral baselines from Stage 6.** Observer pipeline + human comparison.
5. **Eval/search separation enforced.** Tactical terms in search. Strategic terms in eval. Never cross.
6. **No lead penalty in eval.** Broke tactical monotonicity in v1.

**From Stages 10-19:**
7. **EP uses board scan, not player.prev().** `player.prev()` returns eliminated players in 4PC. Use `find_ep_captured_pawn_sq()` pattern from day one.
8. **ArrayVec movegen from Stage 2.** `generate_legal_into(&mut ArrayVec<Move, 256>)` via MoveBuffer trait. No heap allocation per search node.
9. **Single-pass move ordering.** Score all moves into `ArrayVec<(Move, i32), 256>`, sort once. Not separate classification buckets.
10. **LTO + codegen-units=1 from Stage 0.** Free 10-20% release performance.
11. **SIMD planned into NNUE design.** Weight transpose at load time, `align(32)` accumulators, runtime AVX2 detection. Design at Stage 14, not retrofit at Stage 19.
12. **Bitboards: skip.** 14x14 board makes them impractical. Attack query API is sufficient.
13. **Strategy profiles as two axes.** Target selection (vulture/predator/assassin) and play style (fortress/territorial) are independent. Design blend weights for extensibility at Stage 8.
14. **Stress test: volume over depth.** 10K games at depth 2 finds more bugs than 500 at depth 8.

---

## Critical Rules

1. **PLAN FIRST + APPROVAL CHAIN.** Write the plan to `.claude/dispatch_comms.jsonl` (type: "plan", tier: 1). Do NOT execute until Dispatch and user have approved. Plans are living documents; adapt mid-execution if needed, following the same chain.
2. **Depth-4 rule — CORRECTNESS REQUIREMENT.** Only depths divisible by 4 are valid (4, 8, 12...). Non-multiples of 4 are PROHIBITED. Training data at non-multiple-of-4 depths must be discarded.
3. **Game behavior changes require approval.** Any change to move generation, eval, search, rules, scoring, or board representation: write a Tier 2 entry to `.claude/dispatch_comms.jsonl` and wait for explicit user approval before execution.
4. **Write to `.claude/dispatch_comms.jsonl`.** Plans (tier: 1) before execution. Progress, stuck reports, and Tier 2 requests as they arise.
5. **Save point before each stage.** Commit + tag before starting any new stage.
6. **Spot-check outputs.** Don't trust "N records generated" — read actual data. Don't trust "X tests passed" — verify coverage.
7. **Fixed-size data structures in hot paths.** No `Vec<T>` in Board, GameState, or MoveUndo.
8. **Turn order R->B->Y->G.** Never alter outside make/unmake.
9. **Searcher trait FROZEN.** `search(&mut self, &GameState, SearchBudget) -> SearchResult`
10. **Evaluator trait FROZEN.** `eval_scalar` and `eval_4vec`.
11. **Stages aren't done until the user says so** from testing in the UI.
12. **Diagnostic games after every eval/search change.**
13. **Pre-closeout.** Before ending any session: update comms log, run `git status`, verify no untracked work, write a closeout entry.
14. **Cleanup agent follows every session.** Dispatch spins a fresh agent to audit the outgoing agent's work. It reports to Dispatch; it does not fix.

---

## Relationship to Other Projects

- **Project Odin (v1):** The original. All 19 stages complete, lessons extracted. Valhalla is the clean rebuild.
- **Project Freyja:** Sister engine using Max^n with NNUE-guided beam search. Both train own NNUEs via self-play. When complete, they compete head-to-head.

### From Freyja

15. **Gumbel-Top-k MCTS** — replaces UCB1 (2 sim convergence vs 16+)
16. **OMA** — opponent nodes pick 1 move in MCTS (3-4x deeper)
17. **Progressive Widening** — floor(k × visits^α) at root-player nodes
18. **Phase-separated hybrid** — OPPS opening, MCTS midgame
19. **DKW rules** — eliminated pieces as walls, king moves randomly
20. **Ray-attenuated influence maps** — directional piece projection
21. **Observer from Phase 2** — not deferred to last phase
22. **Chain walk** — branchless multiplayer SEE (SUPER-SOMA)

---

## At Session End

Update `masterplan/HANDOFF.md` and `masterplan/STATUS.md`. Create vault notes and a session note per `masterplan/AGENT_CONDUCT_MYTHOS.md`.
