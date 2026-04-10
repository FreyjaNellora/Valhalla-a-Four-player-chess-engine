# Multi-Player MCTS Enhancements

**Citation:** J.P.A.M. Nijssen. "Monte-Carlo Tree Search for Multi-Player Games." PhD Thesis, Maastricht University, 2013.
**Also:** Nijssen & Winands. "Enhancements for Multi-Player MCTS." *CG 2010*, LNCS, Springer, 2011.
**URLs:**
- https://project.dke.maastrichtuniversity.nl/games/files/phd/Nijssen_thesis.pdf
- https://dke.maastrichtuniversity.nl/m.winands/documents/CG2010mp.pdf
- https://dke.maastrichtuniversity.nl/m.winands/documents/pMCTS.pdf

## Progressive History

Combines Progressive Bias with history heuristic. Adds a bias term to UCT selection based on move history scores.

**Key implementation detail:** Denominator divides by `n_i - W_i` (visits minus score = number of LOSSES) not just `n_i`:
- Moves that perform poorly lose bias quickly (denominator grows fast)
- Moves that keep winning retain bias longer (denominator stays small)
- +1 added to denominator to avoid division by zero

**Recommended parameter:** W=5

## MP-MCTS-Solver

Extends MCTS-Solver to multiplayer. Backpropagates proven wins/losses through the tree.

## Results

- Progressive History wins **60-80%** against standard MCTS
- Performance degrades with more players but remains significant
- Dividing by losses is an improvement over dividing by visits

## Games Tested

Focus (2-4 players), Chinese Checkers (3-6 players)

## Relevance to Valhalla

Progressive History is directly applicable to Valhalla's MCTS layer (Phase 4). The W=5 parameter is a good starting point. The finding that performance degrades with more players is a warning — 4-player chess is in the regime where this matters. Consider combining with Gumbel-Top-k.
