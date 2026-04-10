# Multi-Player Static Exchange Evaluation

**Status: NO papers exist on this topic. This is a genuine gap in the literature.**

## What Exists (2-Player Only)

- Chessprogramming Wiki. "Static Exchange Evaluation." Standard 2-player SEE: alternating captures on a single square, lowest-value attacker first.
  - URL: https://www.chessprogramming.org/Static_Exchange_Evaluation

- Vlasov, L. "Static Exchange Evaluation with Alpha-Beta Approach." Applies alpha-beta to the SEE capture chain.
  - URL: https://www.researchgate.net/publication/220174539

## Why the Gap Exists

Multi-player chess variants are niche. In 4-player chess, a capture chain involves up to 4 players with conflicting interests. The 2-player SEE assumption of alternating captures breaks down entirely.

## Key Challenges for 4-Player SEE

1. **Capture order follows 4-player turn order** (R→B→Y→G), not alternating
2. **Each player independently decides** whether to recapture based on own interests
3. **A "bad" capture for player A** might be "good" if it provokes B and C into a fight that benefits A
4. More like a mini Max^n on a single square than simple SEE traversal

## Valhalla's Solution: Chain Walk (Swarm Layer 3)

Adapted from SUPER-SOMA (Rollason 2000):
1. Collect all attackers per player, sorted by piece value (ascending)
2. Follow turn order rotation (R→B→Y→G)
3. Each player: would I trade up or break even? If no: stand pat
4. Continue until all pass or no attackers remain
5. Return material outcome per player: `chain_walk(state, square) -> [i16; 4]`

Properties: branchless, O(n) per contested square, most positions have 2-4 contested squares.

## Relevance

Chain walk is a novel contribution. No published work extends SEE beyond 2 players. The SUPER-SOMA precedent (branchless capture resolution for Shogi) validates the approach of resolving exchanges without tree search.
