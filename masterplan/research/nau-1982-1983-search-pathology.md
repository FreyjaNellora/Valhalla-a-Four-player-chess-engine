# Nau 1982/1983 — Search Pathology

**Citations:**
- Nau, D.S. (1982). "An Investigation of the Causes of Pathology in Games." *Artificial Intelligence*, 19(3):257-278.
- Nau, D.S. (1983). "Pathology on Game Trees Revisited, and an Alternative to Minimaxing." *Artificial Intelligence*, 21(1-2):221-244.
- Mutchler, D. (1993). "The multi-player version of minimax displays game-tree pathology." *Artificial Intelligence*, 64(2):323-336.

**URLs:**
- https://www.cs.umd.edu/~nau/papers/nau1982investigation.pdf
- https://www.cs.umd.edu/~nau/papers/nau1983pathology.pdf
- https://www.sciencedirect.com/science/article/abs/pii/000437029390108N

## The Phenomenon

Deeper minimax search can make play WORSE, not better. Minimax amplifies noise in heuristic evaluation.

## Exact Conditions for Pathology

1. **Independent sibling node values** — when nearby positions' true values are uncorrelated
2. Binary trees with leaf values {+1, -1}
3. High enough constant branching factor combined with the above

## What Prevents Pathology

1. **Strong sibling dependencies** — when nearby positions have correlated values (as in real chess)
2. **Graph structure** (transpositions) — positions reachable by multiple paths create dependencies
3. **Higher evaluation accuracy** at individual nodes

## Multiplayer Extension (Mutchler 1993)

Max^n ALSO displays pathology under the same conditions. Deeper Max^n search can make play worse in multiplayer games.

## Relevance to Valhalla

Swarm is specifically designed to produce correlated sibling values — structural features (material tension, king exposure, etc.) change gradually between sibling positions. The acceptance criteria require measuring sibling correlation > 0.6 at depth 4 and 8.

Also relevant: the depth-4 rule may serve as implicit pathology avoidance — each depth increment completes a full round, ensuring each increase is "meaningful."
