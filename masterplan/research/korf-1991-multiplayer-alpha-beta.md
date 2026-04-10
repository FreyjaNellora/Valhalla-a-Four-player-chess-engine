# Korf 1991 — Multi-Player Alpha-Beta Pruning

**Citation:** Korf, R.E. (1991). "Multi-Player Alpha-Beta Pruning." *Artificial Intelligence*, 48(1):99-111.
**URLs:**
- https://faculty.cc.gatech.edu/~thad/6601-gradAI-fall2015/Korf_Multi-player-Alpha-beta-Pruning.pdf
- https://www.sciencedirect.com/science/article/abs/pii/000437029190082U

## Key Theorems

1. **Without bounds on leaf values, no pruning is possible in Max^n.** Knowing one player's score tells you nothing about others (unlike 2-player zero-sum).

2. **With upper bound on sum of scores (S) and lower bound per player (0), pruning becomes possible.** This is the enabling assumption.

3. **Three pruning classes:**
   - **Immediate:** Prune when child's value equals S (trivial)
   - **Shallow:** Prune based on bounds from siblings at same level. This is the practical multi-player analogue of alpha-beta.
   - **Deep:** Pruning from ancestors multiple levels up is **NOT possible** for n>2 players.

4. **Shallow pruning is optimal** among directional algorithms (Theorem 2).

5. **Best-case effectiveness:** 2-player alpha-beta reduces b to sqrt(b). Multi-player shallow pruning stays close to b — far less dramatic.

## Follow-up Work

- Sturtevant & Korf (2000). Alpha-Beta Branch-and-Bound. URL: https://cdn.aaai.org/AAAI/2000/AAAI00-031.pdf
- Sturtevant (2003). Speculative Pruning. URL: https://www.ijcai.org/Proceedings/03/Papers/098.pdf
- Sturtevant PhD Thesis. URL: https://www.cs.du.edu/~sturtevant/papers/multiplayergamesthesis.pdf

## Relevance to Valhalla

Justifies the paranoid backbone in OPPS. Since deep pruning is impossible for n>2, paranoid reduces to 2-player (enabling full alpha-beta). OPPS parameterizes how much of the paranoid assumption to apply.
