# Phase 1: Board + Rules + Move Generation + DKW

## Commander's Intent

Build the complete, correct, allocation-free board infrastructure for 14x14 four-player chess. Every other phase depends on this being fast and right. Get perft passing, get DKW working, and leave a foundation that never needs to be touched again.

## Reading List

1. `masterplan/MASTERPLAN.md` Section 3 — Phase 1 specification
2. `masterplan/MASTERPLAN.md` Section 4 — Invariants (board + data structure)
3. `masterplan/4PC_RULES_REFERENCE.md` — Board geometry, piece positions, castling paths, EP rules, DKW rules
4. `masterplan/DECISIONS.md` — ADR-001 (Attack Query API), ADR-004 (Fixed-Size Data), ADR-005 (Zobrist Root Player), ADR-013 (EP Board Scan), ADR-014 (ArrayVec Movegen), ADR-016 (Release Profile)
5. `masterplan/SYSTEM_PROFILE.local.md` — Hardware constraints

## Write Scope

- `valhalla-engine/src/board/` — all files
- `valhalla-engine/src/game_state/` — all files
- `valhalla-engine/src/movegen/` — all files
- `valhalla-engine/src/types/` — all files (Move, Square, Piece, Player, etc.)
- `valhalla-engine/src/zobrist/` — all files
- `valhalla-engine/src/dkw/` — all files
- `valhalla-engine/src/lib.rs` — module declarations only
- `valhalla-engine/Cargo.toml` — dependencies (arrayvec)
- `Cargo.toml` — workspace-level release profile
- Tests for all of the above

## Current State

| Field | Value |
|-------|-------|
| **Status** | **COMPLETE** |
| **Last Session** | 2026-04-10 — Full implementation |
| **Blocking Issues** | None |

## Acceptance Checklist

- [x] `perft(1) = 20`, `perft(2) = 395`, `perft(3) = 7,800`, `perft(4) = 152,050`
- [x] make/unmake round-trip preserves identical GameState including Zobrist hash
- [x] No `Vec<T>`, `Box<T>`, or heap allocation in Board, GameState, MoveUndo
- [x] Move generation correct for all four player directions (pawns, castling, EP)
- [x] Dead zone squares never appear in any generated move list
- [ ] Zobrist hash collision rate < expected for hash width (measured on 1M+ positions) — deferred to Phase 2 (needs search to generate 1M+ positions)
- [x] DKW pieces are immovable walls, DKW king generates 1-8 random moves
- [x] EP uses board scan, not player.prev()

## Active Watch Items

- **If perft values don't match:** Do NOT proceed. Perft is the foundation invariant. Debug until exact match.
- **If DKW king random moves cause nondeterminism in tests:** Seed the RNG for test determinism, use true random in play.
- **If move generation exceeds 100ns per position:** Profile immediately (Risk R10). Consider piece-list augmentation.

## Rework Log

| Date | What Changed | Why | Impact |
|------|-------------|-----|--------|
| | | | |

## Downstream Notes

Phase 2 needs:
- `GameState` with `side_to_move()`, `make_move()`, `unmake_move()`
- Attack query API: `is_square_attacked_by()`, `attackers_of()`
- Legal move generation: `generate_legal_into(&mut ArrayVec<Move, 256>)`
- Zobrist hash accessible for TT
- DKW state queryable (which players are eliminated, which squares are walls)
- Board indexing by Square
