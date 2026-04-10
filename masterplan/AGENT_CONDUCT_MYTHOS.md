# AGENT CONDUCT (Mythos) -- Project Valhalla

> **You are reading the Mythos variant.** All constraints, approval gates, correctness rules, and domain knowledge are identical to `AGENT_CONDUCT.md`. Procedural scaffolding has been removed. **Principle: Less procedure. Same knowledge. Same boundaries.**

**Version:** 2.0-Mythos
**Source:** AGENT_CONDUCT.md v2.0

---

## 0. Preamble

| Document | Defines | Authority Over |
|----------|---------|---------------|
| `MASTERPLAN.md` | WHAT each phase builds | Phase specs, acceptance criteria, architecture |
| `4PC_RULES_REFERENCE.md` | The game rules | Board layout, piece movement, scoring, game modes |
| `AGENT_CONDUCT_MYTHOS.md` (this) | HOW agents work | Behavior rules, permissions, communication, code standards |

Phase-specific details live in `masterplan/phases/phase-N.md`.

---

## 1. Information Hierarchy

Look things up in this order. Stop when you find the answer.

1. **Phase reading list** — `masterplan/phases/phase-N.md`
2. **Project docs** — `MASTERPLAN.md`, `DECISIONS.md`, `4PC_RULES_REFERENCE.md`, `STATUS.md`, `HANDOFF.md`
3. **Research library** — `masterplan/research/`
4. **Saved references** — URLs in research files
5. **Web search** — last resort. Sparse direct literature on 4PC. Adjacent fields: multi-agent game theory, RTS AI, influence mapping, MCTS variants. Record findings.

`SYSTEM_PROFILE.local.md` is gitignored. Contains machine specs. Create if missing.

---

## 2. Session Protocol

**Start:** STATUS.md -> phase file -> latest session note -> HANDOFF.md -> verify clean build.

**During:** Write to `.claude/dispatch_comms.jsonl`. Plans before code. Recommend, don't just report. Plans are living documents — adapt when evidence says to.

**End:** Update phase file (Current State, Rework Log, Downstream Notes) -> create session note in `sessions/phase-N/` -> update STATUS.md -> update HANDOFF.md.

**Compression rule:** "Would the next agent need this to do their job?" If no, cut it.

**Pre-closeout:** Re-read AGENT_CONDUCT and CLAUDE. Audit comms log. `git status`. Write closeout entry.

---

## 3. Permission Model

**Read:** Anything.

**Write:** Only within your phase's write scope (defined in `masterplan/phases/phase-N.md`). Always-allowed exceptions: `.claude/dispatch_comms.jsonl`, `STATUS.md`, `HANDOFF.md`, `sessions/`, `issues/`, your own phase file's mutable sections.

**Cross-phase writes:** Change order required (see Section 4).

**All changes require plan + approval.** Claude Code writes plan -> Dispatch evaluates -> user approves -> execution begins.

---

## 4. Cross-Phase Protocol

Discover issue in another phase's territory -> stop ALL work -> create change order in `masterplan/change-orders/` -> log Tier 2 -> send Gmail notification -> kill all background processes -> wait indefinitely for approval -> both phase files get updated. The user may be at work for hours. One notification + one follow-up max. The work waits.

**FRAGO:** Only communicate what's different. Don't re-explain known context.

**Deferred-debt:** Don't carry "wire later" items across 3+ phases. Implement, redesign, or document a concrete trigger.

---

## 5. Communication Protocol

**Frustration is a warning flag.** Stop and ask probing questions. The frustration almost always means a miscommunication, not a capability gap.

**Repeated questions** mean the user is working through something they can see but cannot express. Help them find words for it. Don't keep answering the surface question.

**Verify before claiming.** Read the code before saying what it does. Run the test before saying it passes.

**Recommend, don't just report.** "I recommend X because Y" beats "Options are X, Y, or Z."

---

## 6. Core Rules

### Depth-4 Rule

Only depths divisible by 4 are valid search depths. Correctness requirement. Training data at non-multiple-of-4 depths must be discarded.

### Test-First

Write tests before or alongside implementation. Never after.

### No Extra Features

Build exactly what the phase spec says. If something seems missing, ask.

### Code Standards

`cargo fmt`, `cargo clippy` zero warnings, doc comments on all pub items, default to private.

### Eval/Search Separation

Eval: purely positional (material, PST, king safety, pawn structure). Search: move ordering, pruning, extensions. Swarm: all tactical assessment at leaves. The test: "Does this term model something the search tree and swarm pipeline cannot discover?" If yes -> eval. If no -> search or swarm.

### Fixed-Size Data

No `Vec<T>` in Board, GameState, MoveUndo. Fixed-size arrays, swap-remove, length counters.

### Turn Order

R->B->Y->G. `side_to_move` only modified inside `make_move`/`unmake_move`.

### Frozen Traits

Once defined in Phase 2:
```rust
trait Evaluator: Send + Sync { fn evaluate(&self, state: &GameState) -> Score; }
trait Searcher { fn search(&mut self, state: &GameState, depth: u32) -> SearchResult; }
```
New capabilities via composition, not trait modification.

### Game Behavior Changes

Any change to move generation, eval, search, rules, scoring, or board representation: analyze -> explain -> Tier 2 entry -> wait for approval. This includes bug fixes. Silent behavioral changes invalidate training data.

### Output Verification

Don't trust at face value. Inspect samples. Verify test coverage. When it looks too clean, look closer.

### Debugging Anti-Spiral

Each analysis pass cites something new, or you're spiraling. Write what you know, what you need to know, and design a distinguishing test.

### Save Points

Commit + tag before starting any new phase. `git tag phase-N-save-point`.

### Diagnostics

Only the top-level agent runs the engine. After eval/search changes, run observer diagnostics. Log naming: `{mode}_{profile}_d{depth}_{games}games_{timestamp}.log`.

### Cleanup Agent

Post-session, Dispatch spins a fresh agent to audit. It reports gaps, it does not fix them.

---

## 7. Observability

`tracing` crate. Zero-cost when filtered. `info!` for high-level events, `debug!` for diagnostics, `trace!` for verbose per-node data.

```
RUST_LOG=valhalla_engine=debug    # Development
RUST_LOG=valhalla_engine=info     # Normal
RUST_LOG=valhalla_engine=trace    # Verbose
```

Protocol logging: `setoption name LogFile value <path>`. Max rounds: `setoption name MaxRounds value <n>`.

---

## 8. Audit Procedure

After completing a phase: `audit_log_phase_N.md` (BLOCKING/WARNING/NOTE), `downstream_log_phase_N.md` (API contracts), `cargo test`, `cargo clippy`, observer diagnostics, user approval before tagging.

---

## 9. Approval Tiers

| Tier | Requires | Examples |
|------|----------|---------|
| 0 | Auto-approve | File reads/edits within scope, builds, tests, git commits |
| 1 | Dispatch approves | Config changes, build scripts, test params, branch merges, plans |
| 2 | User approves | Destructive ops, credentials, external side effects, game behavior changes, cross-phase writes |

---

*This document is the behavioral spec for frontier-tier agents working on Project Valhalla.*
