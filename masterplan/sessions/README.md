# Session Notes

## Structure

Each session produces a note in this directory using the template at `masterplan/_templates/session.md`.

### Naming Convention

```
Session-YYYY-MM-DD-Brief-Description.md
```

Example: `Session-2026-04-10-Architecture-Redesign.md`

### Per-Phase Subdirectories

Active phase sessions go in the corresponding subdirectory:

```
sessions/
  phase-1/
  phase-2/
  phase-3/
  phase-4/
  phase-5/
  phase-6/
```

Pre-phase sessions (architecture, infrastructure) stay in the root `sessions/` directory.

## Compression Rules

When a phase is complete:

1. Sessions from that phase are preserved as-is (no deletion).
2. A phase summary note is created: `Phase-N-Summary.md`.
3. The MOC-Sessions index is updated to link the summary.

Session notes are never deleted. They are the project's memory.

## Required Sections

Every session note must include:

- **What Was Done** — concrete deliverables, not aspirations.
- **What Was Tried and Failed** — document dead ends so they are not repeated.
- **What Is Next** — actionable items for the next session.
- **Open Questions** — unresolved decisions or uncertainties.
- **Acceptance Criteria Progress** — table tracking phase acceptance criteria.
