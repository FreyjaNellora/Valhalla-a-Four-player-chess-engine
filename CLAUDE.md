# Project Valhalla -- Agent Orientation

> **Model-Tier Router**
> **Mythos-tier models (when released by Anthropic):**
> Read `CLAUDE_MYTHOS.md` and `masterplan/AGENT_CONDUCT_MYTHOS.md` instead of this file and `AGENT_CONDUCT.md`.
> The Mythos variants contain identical constraints with reduced procedural scaffolding — optimized for models that don't need compensating guardrails.
>
> **All current models (including Opus 4.6, Sonnet 4.6, Haiku 4.5):** Continue reading this file.

A four-player chess engine: OPPS + Swarm + MCTS + NNUE.
Clean rebuild incorporating lessons from Odin v1 (Stages 0-19) and Freyja (Stages 0-17).

## Before You Start

Read these files in order:

1. `masterplan/STATUS.md` -- Where is the project? What stage? What's blocked?
2. `masterplan/HANDOFF.md` -- What was the last session doing? What's next?
3. `masterplan/AGENT_CONDUCT.md` Section 1.1 -- Full stage entry protocol.

If you're new to the project or starting a new stage, also read:
4. `masterplan/DECISIONS.md` -- Why key architectural choices were made.
5. `masterplan/MASTERPLAN.md` -- Full spec (refer to specific sections as needed).

## Quick Understanding (Obsidian Vault)

For **fast lookup**, use the knowledge vault at `masterplan/`:

| You want to know... | Read |
|---|---|
| Big picture navigation | `masterplan/_index/MOC-Project-Valhalla.md` |
| Tier 1 stages, logs, invariants | `masterplan/_index/MOC-Tier-1-Foundation.md` |
| Known issues | `masterplan/_index/MOC-Active-Issues.md` |
| Session history | `masterplan/_index/MOC-Sessions.md` |
| All wikilink targets | `masterplan/_index/Wikilink-Registry.md` |

## What Goes Where -- The Hard Line

| Content | Where | Rule |
|---|---|---|
| Stage specs, acceptance criteria | `masterplan/MASTERPLAN.md` | Authoritative. Never duplicate elsewhere. |
| ADRs, architectural decisions | `masterplan/DECISIONS.md` | Why key choices were made. |
| Agent behavior rules | `masterplan/AGENT_CONDUCT.md` | HOW agents work. |
| Audit logs, downstream logs | `masterplan/` | Formal records per stage. |
| Project state, session handoff | `masterplan/STATUS.md` + `HANDOFF.md` | Update per AGENT_CONDUCT.md 1.14. |
| Implementation knowledge | `masterplan/components/` | How things work at code level. |
| Component relationships | `masterplan/connections/` | How things connect. |
| Session history | `masterplan/sessions/` | Preserved history. |
| Bugs, workarounds | `masterplan/issues/` | Runtime problems. |
| Implementation patterns | `masterplan/patterns/` | Reusable approaches. |

## Key Differences from v1

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

## Critical Rules

1. **PLAN FIRST + APPROVAL CHAIN.** Always start in plan mode. Write the plan to `.claude/dispatch_comms.jsonl` (type: "plan", tier: 1). Dispatch reviews first, then user approves — do NOT execute until both have signed off. Plans are living documents; adapt mid-execution if needed, following the same chain. (AGENT_CONDUCT 1.0, 1.0a, 1.15)
2. **Depth-4 rule — CORRECTNESS REQUIREMENT.** Only depths divisible by 4 are valid (4, 8, 12...). Depth 3 and any non-multiple-of-4 is PROHIBITED. Training data at non-multiple-of-4 depths must be discarded. (AGENT_CONDUCT 1.2)
3. **Game behavior changes require approval.** Any change to move generation, eval, search, rules, scoring, or board representation: analyze → explain → write Tier 2 to `.claude/dispatch_comms.jsonl` → WAIT for user approval. Never silent. (AGENT_CONDUCT 1.14)
4. **Write to `.claude/dispatch_comms.jsonl`.** Write plans (type: "plan", tier: 1) before execution — include what's changing, why, files affected, risks, and verification method. Log periodic work progress, stuck reports, and Tier 2 requests. Dispatch reviews all plans. Stuck reports are informational — keep working. (AGENT_CONDUCT 1.0, 1.15)
5. **Save point before each stage.** Commit + tag before starting any new stage. (AGENT_CONDUCT 1.16)
6. **Spot-check outputs.** Don't trust "N records generated" — read actual data. Don't trust "X tests passed" — verify coverage. (AGENT_CONDUCT 1.17)
7. **Fixed-size data structures in hot paths.** No `Vec<T>` in Board, GameState, or MoveUndo.
8. **Turn order R->B->Y->G.** Never alter outside make/unmake.
9. **Searcher trait FROZEN.** `search(&mut self, &GameState, SearchBudget) -> SearchResult`
10. **Evaluator trait FROZEN.** `eval_scalar` and `eval_4vec`.
11. **Stages aren't done until the user says so** from testing in the UI.
12. **Diagnostic games after every eval/search change.**
13. **Pre-closeout re-read.** Before ending any session: re-read AGENT_CONDUCT.md + CLAUDE.md, self-audit the comms log, run `git status`, verify no untracked work, write a closeout comms entry. Non-negotiable. (AGENT_CONDUCT 1.18)
14. **Cleanup agent follows every session.** Dispatch spins a fresh agent to audit the outgoing agent's work — finds uncommitted files, log gaps, untracked artifacts. It reports to Dispatch; it does not fix. (AGENT_CONDUCT 1.19)

## Relationship to Other Projects

- **Project Odin (v1):** The original. All 19 stages complete, lessons extracted. Valhalla is the clean rebuild.
- **Project Freyja:** Sister engine using Max^n with NNUE-guided beam search. Both engines train their own NNUEs via self-play. When complete, they compete head-to-head.

### Lessons Imported from Freyja

15. **Gumbel-Top-k MCTS.** Converges in 2 simulations vs 16+ for UCB1. Replaces UCB1 at root.
16. **Opponent Move Abstraction (OMA).** Opponents pick 1 move via lightweight policy in MCTS. 3-4x deeper search.
17. **Progressive Widening.** Limit MCTS children as `floor(k × visits^α)` at root-player nodes.
18. **Phase-separated hybrid.** OPPS-only in structured openings, MCTS in chaotic midgame.
19. **DKW rules.** Dead King Walking: eliminated player's pieces become immovable walls, king moves randomly.
20. **Ray-attenuated influence maps.** Pieces project influence along movement vectors with blocker attenuation.
21. **Observer protocol brought forward.** Self-play, A/B testing, and training data extraction from Phase 2, not deferred to last phase.
22. **Chain walk design.** Multiplayer capture resolution without search (SUPER-SOMA adapted for 4 players).

## At Session End

1. Update `masterplan/HANDOFF.md` and `masterplan/STATUS.md`.
2. Create vault notes (issues, components, connections, patterns).
3. Create a session note in `masterplan/sessions/`.
