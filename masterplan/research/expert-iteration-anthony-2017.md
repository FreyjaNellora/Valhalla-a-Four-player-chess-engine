# Expert Iteration (ExIt)

**Citation:** Anthony, T., Tian, Z., & Barber, D. (2017). "Thinking Fast and Slow with Deep Learning and Tree Search." *NeurIPS*, 30.
**URLs:**
- https://arxiv.org/abs/1705.08439
- https://discovery.ucl.ac.uk/id/eprint/10038400/1/Barber_7120-thinking-fast-and-slow-with-deep-learning-and-tree-search.pdf
- PhD Thesis: https://discovery.ucl.ac.uk/id/eprint/10123580/1/ExIt-Thesis-Corrected-0503-2.pdf

## How ExIt Works

Two components:
- **Expert (slow thinking):** Tree search (MCTS). Expensive but accurate.
- **Apprentice (fast thinking):** Neural network policy. Cheap but approximate.

## The Iterative Loop

1. Expert plays games using tree search (guided by current apprentice policy)
2. Collect (state, expert's move distribution) pairs
3. Train apprentice to imitate expert's moves (supervised learning)
4. Use improved apprentice to guide expert's search (better priors = more efficient search)
5. Repeat

## Key Differences from AlphaZero

- ExIt published first (May 2017; AlphaGo Zero October 2017)
- ExIt: supervised learning on expert moves. AlphaZero: additionally uses value learning from game outcomes.
- ExIt explicitly separates "planning" from "generalization"
- Both share core insight: search generates training signal, network speeds up search

## Results

Trained tabula rasa on Hex, defeated MoHex 1.0 (Olympiad champion).

## Extension: BRExIt

"Best Response Expert Iteration" adds opponent modeling.
URL: https://ir.cwi.nl/pub/33351/33351.pdf

## Relevance to Valhalla

ExIt IS what Valhalla's NNUE training does. OPPS+Swarm (expert) generates training data. NNUE (apprentice) learns from it. Trained NNUE improves OPPS leaf evaluation. The training target should be the search policy (which moves search found best), not just game outcome.
