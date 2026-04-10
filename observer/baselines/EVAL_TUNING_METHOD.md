# Valhalla Bootstrap Eval Tuning Method

## Overview

Curated Texel Tuning adapted for 4-player chess. Uses tactical positions extracted
from 3000+ Elo human games to validate and tune eval weights.

**This document is authoritative for eval weight changes. Any agent modifying
eval weights MUST run the test suite and report results.**

**Note:** Valhalla uses OPPS (Opponent Pruning Paranoid Search) with a single-perspective
evaluator. The eval produces a scalar via `evaluate()` relative to the searching player.
The test positions and scoring criteria are derived from real human games. Both Valhalla
and Freyja should converge on the same human-like behavior from the same game data.

## The Method

### Step 1 — Position Set

25 tactical samples in `tactical_samples.json`, each a 5-6 move window from
real games. Each sample has:
- Move sequence to replay (engine notation: `position startpos moves ...`)
- The human move at the decision point
- Expected move category (capture, development, castling, queen_activation, etc.)
- Whether it's a positive or negative example
- What happened as a consequence

### Step 2 — Run the Suite

For each sample where `moves_to_replay` is provided:

```
position startpos moves <replay_moves>
go depth 4
```

Record:
- `bestmove` — what Valhalla chose
- `score` — eval from the `info` line
- Match result (see scoring below)

### Step 3 — Score

| Result | Points | Criteria |
|--------|--------|----------|
| Exact match | +3 | Engine plays the same move as the human |
| Category match | +2 | Different move, same type (any development when human developed, any capture when human captured, any castle when human castled) |
| Reasonable | +1 | Different category but not harmful (e.g., development when human captured — still productive) |
| Neutral | 0 | No clear positive or negative |
| Anti-pattern | -2 | Engine plays a move from the negative examples (pawn spam, knight retreat, queen shuffle, king walk) |
| Blunder | -3 | Engine hangs material or misses a free capture |

For negative examples (is_negative: true), scoring is inverted:
- Engine AVOIDS the bad move: +3
- Engine plays the bad move: -3

### Step 4 — Thresholds

| Score | Verdict | Action |
|-------|---------|--------|
| >= 50 (67%) | PASS | Weights are good. Proceed with other work. |
| 38-49 (50-66%) | MARGINAL | Identify which categories fail. Adjust those specific weights. |
| < 38 (< 50%) | FAIL | Fundamental weight problem. Do not merge. |

Maximum possible score: 75 (25 samples x 3 points each)

### Step 5 — Tuning Loop

When a sample fails:

1. Identify the category (development, castling, king_safety, capture, etc.)
2. Determine which weight controls that behavior (see Weight Map)
3. Adjust the weight by 5-10 units
4. Re-run the full suite
5. Check that the fix doesn't regress other samples
6. Repeat until score >= 50

## Weight Map

Which weight affects which behavior. **Update this table when Phase 2 eval is
implemented — the weight names below are from the MASTERPLAN Phase 2 spec.**

| Behavior | Primary Weight | Secondary |
|----------|---------------|-----------|
| Pieces sitting on back rank | Development bonus (N=45, B=30, Q=50, R=25) | Mobility |
| Queen not activating | Queen development bonus (50) | PST_QUEEN |
| Too many pawn moves | Development bonus (must outweigh pawn advance) | Connected pawn bonus |
| Not castling | King safety (off-home-rank -40cp, pawn shield, open files) | PST_KING |
| Missing captures | Search/move ordering issue, not eval | Move ordering |
| Queen overextension | King safety vs tactical bonus trade-off | King displacement |
| Not promoting passed pawns | Pawn PST + search depth | PST_PAWN high ranks |
| Ignoring king danger | King safety attacker penalty | Pawn shield bonus |

## Planned Weights (From MASTERPLAN Phase 2)

```
Material: P=100, N=500, B=500, R=800, Q=1200, K=10000
Development: Knight=45, Bishop=30, Queen=50, Rook=25
King safety: Off-home-rank displacement = -40cp
             Pawn shield bonus, open file penalty (TBD at implementation)
Connected pawn: Bonus for pawns 2+ ranks past start
PSTs: Gentle gradients, rotated per player
```

**These values will change during Phase 2 implementation. Update this section
when actual weights are finalized.**

## Universal Patterns (From Game Data)

These patterns hold across ALL 12 games in the dataset. They are non-negotiable
requirements for the eval — if the engine violates any of these, weights are wrong.

### Every winner did:
- Queen active by round 5
- Castled (or had deliberate reason not to)
- 2+ captures in first 20 moves
- Pawn ratio <= 35% in opening

### Every checkmated player did:
- Never castled (100% correlation)
- Queen overextended OR dormant
- Pawn ratio > 40% OR front-loaded all pawns before pieces

### Elo calibration targets:

| Metric | 3000+ target | Engine must beat |
|--------|-------------|-----------------|
| Pawn ratio (first 20) | 20-30% | <= 35% |
| Queen activation | Round 2-5 | <= Round 7 |
| Captures in first 20 | 4-5 | >= 2 |
| Knight undevelopment | 0 | 0 |
| Castling | Yes, by R18 | Yes |

## OPPS Considerations

Valhalla uses OPPS (paranoid-family search) where the root player assumes opponents
minimize their score with configurable pruning (n1/l1/l2 parameters). This means:

- **Captures may score differently**: In OPPS, capturing material always looks
  good because opponents are assumed to counter-attack. The n1/l1/l2 parameters
  control how many opponent replies are considered.
- **King safety is even more important**: OPPS assumes the worst case from
  opponents. A cracked king shelter gets punished harder.
- **Development timing may differ**: OPPS can afford slower development because
  it assumes opponents will play anti-you moves regardless.

The test suite measures BEHAVIOR (what move is chosen), not the eval number.
Engines with different search philosophies should produce similar move choices
given good eval weights.

## Game Sources

### Strong Games (tune TOWARD these)
- 95992584: All 3000+ (3054-3434). Red/Yellow tied 73pts.
- 96085550: Mixed (2541-3438). Blue 87pts, checkmate win.
- 93419919: Avg 2931 (2598-3465). Red 79pts.
- 93391655: Avg 3114 (2805-3476). Blue 70pts. Gangster_H wins.
- 93334455: Avg 3165 (2907-3456). Yellow 72pts, Red 79pts.
- 93333795: Avg 3202 (2913-3443). Red 60pts. Gangster_H wins.
- 93060137: Avg 3045 (2902-3422). Yellow 71pts. Gangster_H loses.

### Weak Games (tune AWAY from these)
- 96585003: Weak players show pawn spam, knight undevelopment.
- 96836735: King march to checkmate, queen shuffle.
- 96602063: 54% pawn ratio, queen never activated, 3pts.

### Mixed Games
- 93042775: Red 2803 dominates with bishop pair.
- 93042729: Green (EyeoftheTiger, #1 leaderboard) wins with patient style.
