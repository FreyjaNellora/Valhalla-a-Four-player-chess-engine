# Swarm Intelligence in Game Engines

**Status: NO direct precedent for swarm-style leaf evaluation exists. This is novel territory.**

## What Does Exist

### PSO for Evaluation Weight Tuning (Training, NOT Real-Time Eval)

- Hauptl & Hlavackova-Schindler (2008). "Particle Swarm Optimization Applied to the Chess Game." IEEE CEC 2008. Uses PSO to optimize eval weights (material, mobility, pawn structure, king safety). PSO outperformed simulated annealing.
  - URL: https://ieeexplore.ieee.org/document/4631299/

- Ong et al. (2009). "Evolutionary swarm neural network game engine for Capture Go." ScienceDirect. PSO + neural networks for board eval, training from self-play.
  - URL: https://www.sciencedirect.com/science/article/abs/pii/S089360800900286X

- Abo-Hammour et al. "Co-Evolutionary PSO Applied to 7x7 Seega Game." Two separate PSO swarms co-evolve player evaluation weights.
  - URL: https://ora.ox.ac.uk/objects/uuid:fc430213-a29c-4061-897c-5708b68c79ad

### Ant Colony for Game Tree Exploration

- Kowalski et al. "General Game Playing with Ants." Ant Colony System explores game state space. Each ant represents a player foraging through states.
  - URL: https://link.springer.com/chapter/10.1007/978-3-540-89694-4_39

### Survey

- Pandey et al. (2015). "Role of Particle Swarm Optimization in Computer Games." Springer.
  - URL: https://link.springer.com/chapter/10.1007/978-81-322-2220-0_21

## What Doesn't Exist

- **No papers on swarm-style collective assessment at leaf nodes during search**
- **No multi-agent evaluation at search leaves**
- **No "6-layer tactical pipeline" precedent**

## What Valhalla's Swarm Actually Is

Valhalla's "swarm" is closer to **influence mapping + static exchange evaluation + force-ratio analysis** than to swarm intelligence per se. The name "swarm" captures the multi-layered collective assessment concept but is not derived from PSO/ACO literature.

The closest precedents are:
- SUPER-SOMA (see separate file) — branchless capture resolution
- EBQS (see separate file) — static eval replacing qsearch
- Influence maps (see separate file) — territorial assessment
- Lanchester models (see separate file) — force ratio math

## Relevance to Valhalla

PSO could be useful for tuning swarm layer weights as an alternative to manual calibration. The co-evolutionary approach (separate swarms per player) maps to 4-perspective training. But the swarm pipeline itself (Force Ratio → Pile-On → Chain Walk → Swarm-Delta → Commitment → Participation) is novel and not derived from swarm intelligence literature.
