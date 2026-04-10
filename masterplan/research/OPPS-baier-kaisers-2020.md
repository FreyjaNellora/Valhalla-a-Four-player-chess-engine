# OPPS — Opponent-Pruning Paranoid Search

**Citation:** Hendrik Baier and Michael Kaisers. "Opponent-Pruning Paranoid Search." *FDG '20*, ACM, 2020.
**DOI:** 10.1145/3402942.3402957
**Open access PDF:** https://ir.cwi.nl/pub/30609/30609.pdf

## What It Is

A generalization of BRS+ for fully observable, deterministic multiplayer games. Uses paranoid assumption (all opponents minimize root player's score) to enable alpha-beta pruning. Key innovation: prunes opponent moves to deepen the tree while reducing paranoid pessimism.

## Parameters

Three parameters control opponent move pruning:
- **n1:** Number of opponents allowed the wider move set between your turns (0 to 3)
- **l1:** How many moves those "important" opponents choose from (1 to infinity)
- **l2:** How many moves everyone else gets (1 to 5)
- Constraint: l1 >= l2
- BRS+ is the special case OPPS(1, ∞, 1)

Note: Exact parameter names may differ in the paper — the full paper is behind ACM paywall. The CWI open-access PDF at the URL above should have the definitive definitions.

## Key Findings

- Outperformed BRS+, Max^n, and Paranoid in Chinese Checkers (3, 4, 6 players)
- **Depth parity issue:** OPPS performs differently at even vs odd "rounds" of search. Valhalla's depth-4 rule aligns with favorable parity.
- **rand-Top-k finding:** Randomized opponent move selection from top-k moves improves play quality by preventing degenerate pruning

## Implementation Notes for Valhalla

- Paranoid backbone: root player maximizes, all opponents minimize
- Alpha-beta applies because paranoid reduces to 2-player
- Start with conservative defaults: n1=50000, l1=12, l2=4
- Depth-4 rule: only terminate at depths divisible by 4
- Consider rand-Top-k for opponent move selection

## Relationship to Other Algorithms

- OPPS generalizes BRS+, which generalizes Paranoid
- Max^n is the opposite extreme (no pruning, all players maximize own score)
- MP-Mix (Zuckerman 2009) dynamically switches between strategies; OPPS provides a continuous spectrum via parameters
