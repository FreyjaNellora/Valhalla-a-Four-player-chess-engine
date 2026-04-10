# Speculative Pruning for Max^n

**Citation:** Nathan R. Sturtevant. "Last-Branch and Speculative Pruning Algorithms for Max^n." *IJCAI-03*, 2003.
**URLs:**
- https://www.ijcai.org/Proceedings/03/Papers/098.pdf
- https://webdocs.cs.ualberta.ca/~nathanst/papers/spec_prune.pdf

## What It Is

Pruning algorithms for constant-sum multiplayer games. Extends last-branch pruning to any branch by speculatively assuming the current branch will be at least as good as best seen so far.

## How It Works

**Last-Branch Pruning:** At the last branch of a node, if guaranteed scores for multiple players already exceed maxsum, the branch can be pruned safely.

**Speculative Pruning:** Generalizes to any branch. After searching a child, if sum of all players' best guaranteed scores exceeds maxsum, remaining children can be pruned. Effectiveness depends on move ordering — best moves first = more pruning.

## Branching Factor Reduction

- **Best case:** b → b^((n-1)/n) for n players
- **4-player game:** b^(3/4) — branching factor 30 becomes ~13
- **Chinese Checkers depth 6:** 1.2M → ~100K nodes (**12x reduction**)
- **Hearts/Spades:** +1-3 plies of depth

## Limitations

- Only works for **constant-sum games**. FFA 4-player chess is NOT constant-sum (eliminated players get 0, variable points).
- Effectiveness depends entirely on move ordering quality.

## Relevance to Valhalla

Context for why OPPS is preferred over Max^n for Valhalla. Even with speculative pruning, Max^n's branching reduction (b^0.75) is weaker than OPPS's paranoid reduction (full alpha-beta). This paper is more relevant to Freyja (which uses Max^n) than Valhalla.
