# MP-Mix — Dynamic Strategy Switching

**Citation:** Inon Zuckerman, Ariel Felner, Sarit Kraus. "Mixing Search Strategies for Multi-Player Games." *IJCAI-09*, 2009.
**Extended:** "The MP-MIX Algorithm." *IEEE Trans. CI and AI in Games*, 3(4), 2011.
**URLs:**
- https://www.ijcai.org/Proceedings/09/Papers/113.pdf
- https://ieeexplore.ieee.org/document/6029288/

## What It Is

A meta-algorithm that dynamically switches between three search strategies based on game state.

## The Three Strategies

1. **Max^n:** Each player maximizes own score. Used when no single player is dominant.
2. **Paranoid:** All opponents collude against root player. Used when root player is leading.
3. **Directed Offensive (new):** Root player targets the leader, minimizing THEIR score. Used when another player has a dangerous lead.

## Switching Criteria

Computes "leading edge" — heuristic gap between leader and second place:
- **You ARE the leader (large positive gap):** → Paranoid (protect the lead)
- **Someone else leads (large gap, you're behind):** → Directed Offensive (target the leader)
- **No clear leader (small gap):** → Max^n (play independently)

## Opponent Impact (OI)

Game-specific metric measuring "players' ability to impede opponents." High OI games (Risk — direct attacks) benefit more from strategy switching than low OI games (Hearts — indirect interaction).

## Results

Outperforms pure Max^n AND pure Paranoid in Hearts, Risk, and Quoridor across all settings. Largest improvement in high-OI games.

## Relevance to Valhalla

4-player chess is high-OI (direct captures). MP-Mix's switching logic informs MCTS opponent modeling — Valhalla's 70/20/10 paranoid/max-own/random blend could become position-dependent using leading-edge analysis. Also informs strategy profiles (vulture/predator/assassin): different profiles map to different MP-Mix strategies.
