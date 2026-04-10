# EBQS — Evaluation-Based Quiescence Search

**Citation:** Maarten P.D. Schadd and Mark H.M. Winands. "Quiescence Search for Stratego." *BNAIC 2009.*
**URLs:**
- https://dke.maastrichtuniversity.nl/m.winands/documents/bnaic2009Schadd.pdf
- https://www.semanticscholar.org/paper/Quiescence-Search-for-Stratego-Schadd-Winands/bc1543fcc4af5639c6db0f026584c3036e18abf8

## What It Is

Uses the evaluation function itself to estimate tactical exchange outcomes, instead of expanding a quiescence search tree.

## Why Standard QS Fails in Stratego

1. **Search overhead** — positions don't turn quiet due to chance nodes (imperfect information)
2. **Limited info gain** — QS tree expansion doesn't proportionally improve eval quality

## Results

- EBQS vs no QS: **52.4% win rate** (5,000 games)
- EBQS vs standard QS: **56.3% win rate**
- Standard QS vs no QS: **no improvement** (QS actually hurt due to overhead)

## Key Finding

Standard QS performs WORSE than no QS due to overhead. EBQS improves over both by computing tactical info statically without the tree expansion cost.

## Relevance to Valhalla

Validates the entire swarm approach. In 4-player chess, QS faces the same problems: positions don't quiet with 4 players capturing, and branching makes QS expensive. Swarm is Valhalla's EBQS — static tactical assessment replacing tree-based QS.
