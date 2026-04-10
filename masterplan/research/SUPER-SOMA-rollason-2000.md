# SUPER-SOMA — Branchless Capture Resolution

**Citation:** Jeff Rollason. "SUPER-SOMA — Solving Tactical Exchanges in Shogi without Tree Searching." *Computers and Games (CG 2000)*, LNCS, Springer, 2000.
**Also:** Jeff Rollason. "Looking for Alternatives to Quiescence Search." AI Factory Newsletter, Autumn 2006.
**URLs:**
- https://aifactory.co.uk/downloads/SUPER-SOMA.doc
- http://www.aifactory.co.uk/newsletter/2006_03_quiescence_alts.htm
- https://link.springer.com/chapter/10.1007/3-540-45579-5_19

## What It Is

Branchless capture chain resolution replacing quiescence search. Used in SHOTEST (Shogi engine, placed 3rd worldwide among ~50 programs).

## How It Works

1. **Basic SOMA:** Swap off pieces on a single square. Each side can stop the exchange at their most beneficial moment.
2. **SUPER-SOMA extends to multi-square analysis** via an XREF (cross-reference) table:
   - Scan board for all attacks. Identify pins, ties, discovered attacks.
   - Create XREF entries for every tactical move.
   - Weight by exchange outcome + interaction with other board threats.
   - Select highest-weighted move, update board, repeat until both sides pass.
3. Handles pins/ties, discovered attacks, promotions, defensive moves, mate threats.

## Performance

- **50x fewer nodes** than rival program YSS in same time
- **20x smaller search trees** than competitors
- Move prediction accuracy: **65%**
- Average lookahead: **4.2 plies** in primary continuation
- Extends search depth from 6 to **10.2 plies** at 3-second time limits

## Limitations

- Single-line analysis: early move selection mistakes hard to correct
- Secondary captures only one level deep
- Prioritizes "probability over purity"

## Relevance to Valhalla

Direct precedent for chain walk (swarm Layer 3). In 4-player chess with expensive branching, branchless capture resolution is even more valuable than in Shogi. The XREF concept maps to using influence grid data to identify contested squares.
