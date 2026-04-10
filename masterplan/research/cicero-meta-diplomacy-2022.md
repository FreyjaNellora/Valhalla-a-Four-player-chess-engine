# CICERO — Meta AI Diplomacy

**Citation:** FAIR et al. (2022). "Human-Level Play in the Game of Diplomacy by Combining Language Models with Strategic Reasoning." *Science*, 378(6624):1067-1074.
**URLs:**
- https://www.science.org/doi/10.1126/science.ade9097
- https://ai.meta.com/blog/cicero-ai-negotiates-persuades-and-cooperates-with-people/
- https://github.com/facebookresearch/diplomacy_cicero
- Full technical report: https://noambrown.github.io/papers/22-Science-Diplomacy-TR.pdf

## Architecture

Two coupled modules:
- **Planning engine:** Predicts opponents' likely actions, selects optimal action. Uses piKL algorithm.
- **Dialogue model:** 2.7B parameter LM generates natural language conditioned on strategic intents.

## piKL Algorithm

- Starts from policy prior (learned from 125,261 human games)
- Iteratively updates predictions of each player's actions
- KL-regularized: stays "close" to human-like play while optimizing
- Incorporates dialogue context

## Commitment Handling

Diplomacy has no binding agreements — any player can break promises. CICERO:
- Models what players will ACTUALLY do (based on board + conversation), not what they promise
- Was designed to be relatively honest (filtered to avoid blatant lies)
- This paradoxically made humans trust and cooperate with it more

## Results

Top 10% of players who played >1 game. 2x average score of opponents.

## Relevance to Valhalla

1. **Intent prediction** — predicting what each opponent will actually do, not assuming optimal (Max^n) or worst-case (Paranoid). Maps to MCTS opponent modeling.
2. **piKL** — staying close to human-like play while optimizing. Analogous to NNUE-guided search staying close to evaluated-best moves.
3. **Commitment dynamics** — insight that modeling actual behavior (not optimal behavior) improves play. Relevant to the 70/20/10 paranoid/max-own/random opponent blend.
