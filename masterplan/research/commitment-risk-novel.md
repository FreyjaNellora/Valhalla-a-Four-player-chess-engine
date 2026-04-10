# Commitment Risk in Game Trees

**Status: NO formal framework exists. This is genuinely novel territory.**

## What Does Exist

### Real Options Theory (Economics/Finance)

The closest theoretical framework. Core concepts:
- **Option value** = value of keeping a decision open
- **Exercise** = committing to an irreversible action
- **Time value** = benefit of delaying commitment
- **"Option Games"** combine real options with game theory for strategic decisions under uncertainty

Source: McKinsey. "Option Games: Filling the Hole in the Valuation Toolkit."
URL: https://www.mckinsey.com/~/media/McKinsey/Business%20Functions/Risk/Our%20Insights/Option%20games%20filling%20the%20hole%20in%20the%20valuation%20toolkit%20for%20strategic%20investment/Option%20games.pdf

### Chess Pawn Structure (Implicit Irreversibility)

- Pawns cannot move backwards — pawn moves permanently alter structure
- Treated as heuristic knowledge in eval functions, not formally as "irreversibility penalty"
- No paper formalizes pawn structure evaluation as option value
- URL: https://www.chessprogramming.org/Pawn_Structure

### Game Theory Commitment Value

- "Value of commitment" = difference between payoff from committing vs not committing
- In extensive-form games, commitment limits future options
- Can paradoxically HELP (making threats credible) or HURT (eliminating flexibility)
- URL: https://en.wikipedia.org/wiki/Extensive-form_game

## What Would Need to Be Invented

A formal commitment risk framework for game tree search would need to:
1. **Classify moves by reversibility** — pawn moves, captures = irreversible; piece retreats = reversible
2. **Assign option-value bonuses** to positions with more reversible alternatives
3. **Penalize premature commitment** — committing material to contested squares when you could wait
4. **Handle 4-player amplification** — in 4PC, three opponents can exploit your commitment before you move again

## How Freyja Currently Handles It

Commitment risk in Freyja's eval (inside `compute_swarm()`):
- **Overextension ratio:** fraction of total material on contested squares. High ratio = overextended.
- **Pile-on detection:** when 3+ players converge, the two most committed are penalized, the lurker is rewarded.
- Uses Lanchester square law for the pile-on math.

## Relevance to Valhalla

Commitment counting is swarm Layer 5. Freyja's implementation provides the starting point. The novel contribution is formalizing this within the 6-layer swarm pipeline where commitment risk interacts with chain walk results (Layer 3) and swarm-delta (Layer 4) to drive the participation decision (Layer 6).
