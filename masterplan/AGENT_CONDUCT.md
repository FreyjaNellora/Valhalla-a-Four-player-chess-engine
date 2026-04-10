# AGENT CONDUCT -- AI/Agent Development Rules for Project Valhalla

**Version:** 2.0
**Created:** 2026-02-27
**Revised:** 2026-04-10

---

## 0. Preamble

This document defines HOW AI agents behave while building Project Valhalla. It is one of three core reference documents:

| Document | Defines | Authority Over |
|----------|---------|---------------|
| `MASTERPLAN.md` | WHAT each phase builds | Phase specs, acceptance criteria, architecture |
| `4PC_RULES_REFERENCE.md` | The game rules | Board layout, piece movement, scoring, game modes |
| `AGENT_CONDUCT.md` (this) | HOW agents work | Behavior rules, permissions, communication, code standards |

Phase-specific details (reading lists, write scopes, acceptance checklists, watch items) live in `masterplan/phases/phase-N.md`. This document defines the universal operating framework.

---

## 1. Information Hierarchy

When you need to understand something, look it up in this order. Stop as soon as you find the answer.

| Level | Source | Contains |
|-------|--------|----------|
| 1 | Your phase's reading list | Phase file in `masterplan/phases/`. Lists every file you need for this phase. |
| 2 | Project docs | `MASTERPLAN.md`, `DECISIONS.md`, `4PC_RULES_REFERENCE.md`, `STATUS.md`, `HANDOFF.md` |
| 3 | Research library | `masterplan/research/`. Pre-fetched paper summaries with implementation notes. |
| 4 | Saved references | URLs in research files. Follow only if the summary is insufficient. |
| 5 | Web search | Last resort. Four-player chess has sparse direct literature. Look to adjacent fields: multi-agent game theory, RTS AI, influence mapping, MCTS variants. Record findings in the research library. |

**SYSTEM_PROFILE.local.md** is a gitignored, machine-specific file in `masterplan/`. It contains CPU, RAM, GPU specs and their implications for build times, memory budgets, and feature feasibility. If it does not exist, ask the user for system specs and create it. Never commit it.

---

## 2. Session Protocol

### Session Start

1. Read `masterplan/STATUS.md` — where is the project?
2. Read your phase file in `masterplan/phases/` — what is the current state, what is the acceptance checklist, what are the watch items?
3. Read the latest session note in `masterplan/sessions/phase-N/` — what did the last agent do, what did they leave?
4. Read `masterplan/HANDOFF.md` — any cross-cutting context?
5. Verify clean build: `cargo build --release && cargo test` must pass before you start.
6. After Phase 1: verify perft(1)=20, perft(2)=395, perft(3)=7800, perft(4)=152,050.

### During Session

Write to `.claude/dispatch_comms.jsonl` throughout all work.

**Schema:**
```json
{"timestamp": "ISO8601", "source": "claude-code|dispatch", "type": "status|stuck|request|approval|instruction|work-log|plan|extension", "tier": 0|1|2, "message": "...", "resolved": false}
```

**Plans:** Before writing any code or making any change, write the plan to `.claude/dispatch_comms.jsonl` with `type: "plan"`, `tier: 1`. Include: what is changing, why, files affected, risks, verification method. Do NOT execute until Dispatch and user have approved.

**Work log:** Write progress notes at each meaningful decision point or after each failed attempt. Recommend, don't just report — opinionated entries are more useful than neutral ones.

**Stuck reports:** If stuck for more than 10 minutes or 3 failed attempts, write a `type: "stuck"` entry. Informational only — keep working. Do NOT context-switch unless Dispatch explicitly instructs it.

**Plans are living documents.** If something isn't working mid-execution, flag it in comms and adapt. The goal is the outcome, not adherence to a specific plan. Plan changes follow the same approval chain.

**Check-in timing:** Adjust frequency based on current state:
- Waiting on approval: check every 2 minutes
- Normal work: check every 10 minutes
- Just received a response: check every 5 minutes
- Deep in long task (build/games running): check every 15 minutes
- Need more time? Write an extension ticket: `{"type":"extension","source":"claude-code","tier":0,"message":"EXTENSION REQUEST: [reason]. Will check back by [time/condition].","resolved":false}`

### Session End

1. **Update the phase file** (`masterplan/phases/phase-N.md`): update Current State table, add any rework log entries, update downstream notes if anything changed.
2. **Create a session note** in `masterplan/sessions/phase-N/`. Format: `Session-YYYY-MM-DD-Brief-Description.md`. Include: what was done, what was tried and failed, what is next, any open questions.
3. **Update `masterplan/STATUS.md`** with current state.
4. **Update `masterplan/HANDOFF.md`** with cross-cutting context for the next agent.
5. **Compression rule:** Before writing any session artifact, ask: "Would the next agent need this to do their job?" If no, cut it.

**Pre-closeout checklist (mandatory):**
1. Re-read AGENT_CONDUCT.md and CLAUDE.md in full.
2. Self-audit against the comms log — anything promised but not delivered?
3. Run `git status`. Flag uncommitted changes.
4. Check for untracked files that should be committed or logged.
5. Write a closeout entry to the comms log.

---

## 3. Permission Model

### Read Permissions

Read anything in the project. No restrictions.

### Write Permissions

Write only within your phase's write scope, as defined in `masterplan/phases/phase-N.md`.

Exceptions (always allowed):
- `.claude/dispatch_comms.jsonl` — comms log
- `masterplan/STATUS.md` — status updates
- `masterplan/HANDOFF.md` — handoff notes
- `masterplan/sessions/` — session notes
- `masterplan/issues/` — issue tracking
- `masterplan/phases/phase-N.md` — your own phase file (Current State, Rework Log, Downstream Notes sections only)
- Test files within your write scope

### Cross-Phase Writes

Any change to files outside your write scope requires a change order (see Section 4).

### All Changes Require Plan + Approval

No code is written without a plan in `.claude/dispatch_comms.jsonl`. Approval chain: Claude Code writes plan -> Dispatch evaluates -> Dispatch relays to user -> user approves -> execution begins.

### What NEVER Goes to GitHub

These files are gitignored for a reason. Never commit, push, or include them in PRs:

| File | Why |
|------|-----|
| `masterplan/NOTIFICATIONS.local.md` | Contains ntfy topic name (treat as secret) |
| `masterplan/SYSTEM_PROFILE.local.md` | Machine-specific hardware specs |
| `.claude/settings.local.json` | Local Claude config |
| `.claude/dispatch_comms.jsonl` | Session-specific work log |
| `.env`, `.env.local` | Environment variables / secrets |
| Any API tokens, keys, or credentials | Obvious |
| ntfy topic names, Gotify tokens | Notification channel = secret |

**Rule:** If a file has `.local` in its name or is listed in `.gitignore`, it does not go to GitHub. Period. If you're unsure whether something is sensitive, ask — don't push.

---

## 4. Cross-Phase Protocol

When you discover that your phase needs a change in another phase's territory:

1. **Stop.** Do not make the change. Do not continue other work.
2. **Create a change order** in `masterplan/change-orders/`. Use the template in `masterplan/change-orders/README.md`.
3. **Log a Tier 2 entry** to `.claude/dispatch_comms.jsonl` referencing the change order.
4. **Send Gmail notification** to the user (see Section 5 — HARD STOP protocol).
5. **Kill all background processes.** Nothing runs while waiting.
6. **Wait for approval.** Indefinitely.
7. Once approved, both phase files get updated (the requesting phase and the target phase).

**FRAGO principle:** Only communicate what's different from the existing plan. Don't re-explain context the receiving phase already knows.

**Deferred-debt escalation:** If something is specified in phase N but deferred until phase M (where M > N+2), re-evaluate: implement now if feasible, redesign if the deferral is architecturally wrong, or document with a concrete trigger. Do NOT carry "wire later" items across 3+ phases.

---

## 5. Communication Protocol

**Frustration is a warning flag.** When the user seems frustrated, STOP what you are doing and ask probing questions. The frustration almost always means a miscommunication, not a capability gap. Find the disconnect before continuing.

**Repeated questions mean the user is working through something they can see but cannot express.** Help them find words for it. Don't keep answering the surface question — dig into what they are actually reaching for.

**Verify before claiming.** Read the code before saying what it does. Run the test before saying it passes. Build before saying it compiles. Claims without verification erode trust faster than any bug.

**Recommend, don't just report.** Opinionated entries are more useful than neutral ones. "I recommend X because Y" beats "Options are X, Y, or Z."

**ntfy Notifications (MANDATORY).** Agents MUST send an ntfy notification for ALL of the following events. Read `masterplan/NOTIFICATIONS.local.md` for the topic name and curl format.

| Event | Title Format | Priority |
|-------|-------------|----------|
| Phase complete | `[COMPLETE] Phase N: Brief summary` | 4 (high) |
| Tier 2 approval needed | `[TIER 2] Phase N: What needs approval` | 5 (urgent) |
| Session ending | `[SESSION END] Phase N: What was done` | 3 (normal) |
| Blocked / broken | `[BLOCKED] Phase N: What broke` | 4 (high) |
| Milestone reached | `[MILESTONE] Phase N: What was achieved` | 3 (normal) |

The user may be away from their desk. ntfy is how they stay informed. Do not skip notifications — if in doubt, send one.

**HARD STOP on Tier 2 requests.** When a Tier 2 approval is needed:
1. **ALL work stops.** Not "continue other tasks while waiting." FULL STOP.
2. **ALL background processes stop.** Kill training runs, self-play, builds — everything.
3. **Send notification** via ntfy:
   ```
   curl -s -H "Title: [TIER 2] Phase N: Brief description" -d "SITUATION: ...\nRECOMMENDATION: APPROVE/DENY\nIMPACT IF DELAYED: ..." ntfy.sh/<TOPIC> (see masterplan/NOTIFICATIONS.local.md for topic name)
   ```
4. **Poll for response** using a loop. Check ntfy for replies:
   ```
   curl -s "ntfy.sh/<TOPIC> (see masterplan/NOTIFICATIONS.local.md for topic name)/json?poll=1&since=30m"
   ```
5. **When the user responds** (visible as a message in the poll results), resume from where you stopped.

**Check-in schedule (user works 12-8:30pm EST Sun-Thu):**
- User's break windows: ~2pm, ~4pm (30min lunch), ~6:15pm
- As a break approaches: check every 10-15 minutes
- During break windows: check every 5 minutes
- If no response during break: fall back to every 30 minutes until next break
- Outside work hours: check every 15-20 minutes (user is more available)
- **Be patient.** Send notifications as needed but understand the user may not be able to reply for hours. The work waits.

**Two-way communication:** The ntfy topic `Valhalla-Dispatch-Nell_From_Hell` is bidirectional. Agent sends via curl, user replies from ntfy app on their phone. Agent reads responses by polling the topic.

---

## 6. Core Rules

### 6.1 Search Depth Policy

**Only depths divisible by 4 are valid search depths.** In four-player chess, each player takes one ply per round. A non-multiple-of-4 depth creates evaluation bias — the last-moving player gets an artificial advantage because opponents don't get to respond.

- **Depth 4:** Minimum acceptable. One full round.
- **Depth 8:** Maximum practical given current hardware.
- **Depth 12+:** Not feasible on current hardware. Do not target.
- **Depths 1, 2, 3, 5, 6, 7:** Never use as a final search depth. The only exception is internal iterative deepening loops stepping toward a depth-4/8 target — intermediate results must never be treated as final.

**Training data must be filtered:** Position records at non-multiple-of-4 depths must be discarded. Data at depth 3 is structurally biased and will corrupt the training distribution.

This is a correctness requirement, not a performance preference.

### 6.2 Test-First Development

Write tests before or alongside implementation, never after. Each build-order step should have tests verifying it works before moving to the next step.

### 6.3 No Extra Features

Build exactly what the phase spec says. No bonus features, no "while I'm here" improvements, no forward-looking abstractions. If something seems missing from the spec, ask — don't assume.

### 6.4 Code Standards

- `cargo fmt` before every commit
- `cargo clippy` — zero warnings
- Doc comments on all public items
- Default to private; expose only what downstream needs
- Every `pub` item is a contract

### 6.5 Eval/Search Separation

**Enforced boundary.** Do not cross this line:

| Belongs in EVAL | Belongs in SEARCH |
|----------------|-------------------|
| Material counting | Move ordering (MVV-LVA, SEE) |
| Piece-square tables | Capture extension (via swarm stability) |
| King safety (pawn shield, open files) | Killer/history heuristics |
| Pawn structure | Progressive narrowing |
| Development bonuses | Alpha-beta pruning |
| Score/point awareness | OPPS opponent pruning |

**Updated for OPPS+Swarm:** Swarm handles all tactical assessment at leaves. Eval handles purely positional assessment. Neither contains the other's concerns. The test: "Does this term model something the search tree and swarm pipeline cannot discover?" If yes, add to eval. If no, it belongs in search or swarm.

### 6.6 Fixed-Size Data Rule

No `Vec<T>` in Board, GameState, or MoveUndo. Fixed-size arrays with length counters and swap-remove semantics in hot paths. `Vec` is fine in non-hot paths (move lists from `generate_legal_moves`, PV lines, etc.).

### 6.7 Turn Order Invariant

Turn order is R->B->Y->G. `side_to_move` is ONLY modified inside `make_move`/`unmake_move`. Never alter it directly.

### 6.8 Frozen Traits

Once defined in Phase 2, the Evaluator and Searcher traits are never modified:

```rust
pub trait Evaluator: Send + Sync {
    fn evaluate(&self, state: &GameState) -> Score;
}

pub trait Searcher {
    fn search(&mut self, state: &GameState, depth: u32) -> SearchResult;
}
```

New capabilities are added via composition, not trait modification.

### 6.9 Game Behavior Change Reporting

Any change that could alter how the game behaves requires analysis, explicit reporting, and user approval BEFORE execution.

**What counts:** Move generation, evaluation logic, search behavior, game rules, scoring, piece movement, board representation.

**Procedure:** Analyze the change. Write a plain-language explanation (current behavior, new behavior, expected gameplay impact). Write a Tier 2 entry to `.claude/dispatch_comms.jsonl`. Wait for user approval.

This applies to bug fixes. Correctness fixes are the most important category to report. Silent behavioral changes invalidate prior self-play data and training sets.

### 6.10 Output Verification

Don't trust script output at face value. After generating a data batch, inspect a sample — verify depths, positions, and value ranges. After `cargo test`, verify tests cover intended behavior. When something looks too clean, look closer.

### 6.11 Debugging Anti-Spiral

Each analysis pass must cite something NEW (a new file read, a new test result, a new hypothesis). If repeating the same observations, stop and:
1. Write down exactly what you know
2. Write down what you need to know
3. Design a test that distinguishes the hypotheses

### 6.12 Save Point Protocol

Before starting any new phase: verify `cargo build --release && cargo test` passes, commit all pending changes, tag the commit (`git tag phase-N-save-point`), record the tag in STATUS.md.

### 6.13 Diagnostic Gameplay

**Who runs diagnostics:** Only the top-level orchestrating agent may start the engine, build the project, or run diagnostic games. Subagents MUST NOT independently start/stop the engine.

**When to run:** After any eval change, after any search change, before/after phase completion, when unexpected behavior is reported.

**Workflow:** Build -> configure observer -> run `node observer/observer.mjs` -> review outputs -> compare against baselines -> if regression, create issue and investigate.

**Log naming:** `{mode}_{profile}_d{depth}_{games}games_{timestamp}.log`

### 6.14 Cleanup Agent Protocol

After any agent completes a session, Dispatch spins a fresh agent (clean context) to audit the outgoing agent's work. It reviews comms log entries, runs `git status` and `git diff`, scans for untracked files, verifies the comms log is current, and reports gaps to Dispatch. It does not fix — it reports.

---

## 7. Observability

The engine uses the `tracing` crate from day one. No custom telemetry.

### 7.1 Tracing Usage

- `tracing::info!` — High-level events (search start/complete, position set, game events)
- `tracing::debug!` — Diagnostic output (eval breakdown, TT hits, move ordering)
- `tracing::trace!` — Verbose per-node data (only in development)

All tracing calls are zero-cost when filtered out at runtime.

### 7.2 Environment Configuration

```
RUST_LOG=valhalla_engine=debug    # Development
RUST_LOG=valhalla_engine=info     # Normal operation
RUST_LOG=valhalla_engine=trace    # Verbose debugging
```

**Engine protocol logging:**
- Enable: `setoption name LogFile value <path>`
- Disable: `setoption name LogFile value none`
- Format: `> incoming_command` and `< outgoing_response` per line, timestamped
- Zero overhead when disabled

**Max Rounds:** `setoption name MaxRounds value <n>` stops after N rounds. 20 rounds (80 ply) is usually sufficient for behavioral patterns.

---

## 8. Audit Procedure

After completing a phase:
1. Write `audit_log_phase_N.md` with findings categorized as BLOCKING/WARNING/NOTE
2. Write `downstream_log_phase_N.md` with API contracts and notes for future phases
3. Run `cargo test` and `cargo clippy`
4. Run diagnostic games via observer
5. Get user approval before tagging

---

## 9. Approval Tiers

| Tier | Requires | Examples |
|------|----------|---------|
| 0 | Auto-approve | File reads, edits within write scope, builds, tests, cargo commands, git commits to feature branches |
| 1 | Dispatch approves | Config changes, build script modifications, test parameter adjustments, branch merges, plans |
| 2 | User approves | Destructive operations, credential/env changes, external side effects, game behavior changes, cross-phase writes |

---

*This document is the behavioral spec for all agents working on Project Valhalla.*
