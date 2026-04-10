# Change Orders

## Purpose

Change orders handle cross-phase modifications — when work in one phase requires changing something that was completed in an earlier phase.

## When to Use

A change order is required when:
- A Phase 3 discovery requires modifying Phase 1 board representation.
- A Phase 5 NNUE requirement changes the eval trait from Phase 2.
- Any modification to an already-accepted phase's code or spec.

A change order is NOT required for:
- Normal work within the current phase.
- Bug fixes to the current phase.
- Documentation updates.

## Template

```markdown
# Change Order CO-NNN

**Date:** YYYY-MM-DD
**Source Phase:** (phase requesting the change)
**Target Phase:** (phase being modified)
**Status:** PROPOSED | APPROVED | IMPLEMENTED | REJECTED

## What Needs to Change

(Specific files, functions, or interfaces that need modification)

## Why

(What new information or requirement drives this change)

## Impact Assessment

- Files affected:
- Tests affected:
- Risk of regression:

## Implementation Plan

(Step-by-step plan for making the change safely)

## Verification

(How to confirm the change works and nothing regressed)
```

## Naming Convention

```
CO-001-brief-description.md
```

## Process

1. Author writes the change order with PROPOSED status.
2. Write to `.claude/dispatch_comms.jsonl` as a Tier 2 request.
3. User reviews and approves or rejects.
4. If approved, implement and update status to IMPLEMENTED.
5. Run all affected phase tests to verify no regression.
