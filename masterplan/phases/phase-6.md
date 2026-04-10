# Phase 6: UI + Full Integration

## Commander's Intent

Build the desktop application that lets humans play against the engine and analyze its thinking. The UI must render the 14x14 board correctly, support all four player seats (human or engine), and display real-time search telemetry. This is where the engine becomes a product.

## Reading List

1. `masterplan/MASTERPLAN.md` Section 3 — Phase 6 specification
2. `masterplan/phases/phase-2.md` — Downstream notes (observer protocol, trait interfaces)
3. `masterplan/phases/phase-5.md` — Downstream notes (engine config, eval breakdown)
4. `masterplan/SYSTEM_PROFILE.local.md` — Display, performance constraints

## Write Scope

- `valhalla-ui/` — all files (TypeScript + React + Tauri)
- `valhalla-engine/src/protocol/` — protocol extensions for UI communication (JSON over stdin/stdout or socket)
- Tauri configuration and build scripts
- Tests for UI components

## Current State

| Field | Value |
|-------|-------|
| **Status** | not-started |
| **Last Session** | -- |
| **Blocking Issues** | Phase 2 needed for API contract; Phase 4+ for full integration |

## Acceptance Checklist

- [ ] Board renders correctly for all four players
- [ ] Human can play against engine
- [ ] Analysis panel shows real-time search info
- [ ] Observer telemetry displays without dropped messages
- [ ] UI responds within 16ms (60fps) during engine computation

## Active Watch Items

- **If Tauri sidecar communication is unreliable:** Fall back to local TCP socket. Protocol is JSON, transport is replaceable.
- **If 14x14 board rendering is slow:** Pre-render dead zones. Batch draw calls. Profile canvas vs SVG vs DOM approaches.
- **If analysis panel floods with data:** Throttle update frequency. Buffer and batch telemetry messages.

## Rework Log

| Date | What Changed | Why | Impact |
|------|-------------|-----|--------|
| | | | |

## Downstream Notes

This is the final phase. No downstream consumers.

Post-completion:
- Valhalla vs Freyja head-to-head duels
- Strategy profile tuning via self-play tournaments
- NNUE generational improvement cycles
