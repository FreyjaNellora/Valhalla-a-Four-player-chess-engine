# Influence Maps in Game AI

**Key Sources:**
- Mark, D. (2015). "Modular Tactical Influence Maps." Game AI Pro 2, Ch. 30. URL: https://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter30_Modular_Tactical_Influence_Maps.pdf
- Lewis, M. (2015). "Escaping the Grid: Infinite-Resolution Influence Mapping." Game AI Pro 2, Ch. 29. URL: http://www.gameaipro.com/GameAIPro2/GameAIPro2_Chapter29_Escaping_the_Grid_Infinite-Resolution_Influence_Mapping.pdf
- Hagelback & Johansson. "The Rise of Potential Fields in RTS Bots." URL: https://www.researchgate.net/publication/30498947
- GameDev.net. "The Core Mechanics of Influence Mapping." URL: https://www.gamedev.net/tutorials/programming/artificial-intelligence/the-core-mechanics-of-influence-mapping-r2799/
- Chessprogramming Wiki. "Attack and Defend Maps." URL: https://www.chessprogramming.org/Attack_and_Defend_Maps

## How Influence Maps Work

- **Propagation** = placement (source values at piece locations) + diffusion (spreading to neighbors)
- **Decay:** Exponential falloff from source, applied per propagation step
- **Double-buffering:** Old values in read buffer, new in write buffer to avoid order-dependent artifacts
- **Combination:** Multiple maps layered additively or multiplicatively (my influence minus enemy influence = territory control)
- **BFS flood-fill** for discrete grids; continuous potential fields for non-grid

## Key Implementation Details (from Game AI Pro 2)

- Modular: separate maps for attack, defense, king safety, then combine
- Decay functions: linear, exponential, or custom per piece type
- Update frequency: can be incremental (only update changed regions) for performance

## Relevance to Valhalla

Directly applicable. The 14x14 board is a natural grid. Freyja already has ray-attenuated influence maps (ADR-020) which are carried forward to Valhalla. These feed swarm Layers 1-3 (force ratio, pile-on detection, chain walk).
