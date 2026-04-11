# Research Reference Library

Pre-fetched summaries of academic papers relevant to Valhalla's architecture. **Check here BEFORE searching online** — most implementation-relevant details are already captured.

## Search & Pruning
| File | Topic |
|------|-------|
| `OPPS-baier-kaisers-2020.md` | Opponent-Pruning Paranoid Search — Valhalla's search algorithm |
| `korf-1991-multiplayer-alpha-beta.md` | Why deep pruning is impossible for n>2 players — justifies OPPS |
| `speculative-pruning-sturtevant-2003.md` | Max^n pruning — context for why OPPS is preferred |
| `MP-Mix-zuckerman-2009.md` | Dynamic strategy switching — informs opponent modeling |
| `nau-1982-1983-search-pathology.md` | When deeper search makes play WORSE — swarm must prevent this |

## Tactical Resolution (Swarm)
| File | Topic |
|------|-------|
| `SUPER-SOMA-rollason-2000.md` | Branchless capture resolution — precedent for chain walk (Layer 3) |
| `EBQS-schadd-winands-2009.md` | Static eval replacing qsearch — validates swarm approach |
| `swarm-intelligence-game-engines.md` | PSO for weight tuning; NO leaf-eval precedent exists (novel) |
| `commitment-risk-novel.md` | Irreversibility in game trees — no formal framework exists (novel) |
| `multiplayer-see-novel.md` | 4-player capture resolution — no papers exist (novel) |

## Evaluation Infrastructure
| File | Topic |
|------|-------|
| `influence-maps-game-ai.md` | Territorial control, BFS propagation, decay — feeds swarm |
| `lanchester-attrition-models.md` | Force ratio math — pile-on detection (Layer 2) |

## MCTS & Training
| File | Topic |
|------|-------|
| `MCTS-multiplayer-nijssen-2013.md` | Progressive History, MP-MCTS-Solver |
| `alphazero-silver-2018.md` | Self-play co-development cycle |
| `expert-iteration-anthony-2017.md` | Search as expert, network as apprentice |
| `cicero-meta-diplomacy-2022.md` | Opponent intent modeling in 7-player game |

## Opponent Modeling & Uncertainty
| File | Topic |
|------|-------|
| `fog-of-intention-opponent-inference.md` | Bayesian opponent intent — poker, Diplomacy, RTS, Go analogues (novel for board games) |

## Novel Contributions (No Prior Art)
These topics have NO direct academic precedent. The files document what adjacent work exists:
- Swarm-style leaf evaluation pipeline
- Commitment risk / irreversibility in game tree search (AKA "opportunity cost of attention")
- Multi-player Static Exchange Evaluation (chain walk)
- Fog of intention / Bayesian opponent inference for multiplayer board games

## Research TODOs
Phase-tagged research tasks that need investigation: see `../to-be-researched/README.md`
