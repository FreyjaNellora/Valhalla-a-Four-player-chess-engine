# Lanchester Attrition Models

**Key Sources:**
- Stanescu, Barriga, & Buro (2015). "Using Lanchester Attrition Laws for Combat Prediction in StarCraft." AIIDE-15. URL: https://ojs.aaai.org/index.php/AIIDE/article/view/12780
- IEEE (2022). "Advanced Lanchester Combat Model for Inhomogeneous Armies in RTS Games." URL: https://ieeexplore.ieee.org/document/9707867/
- Stanescu et al. (2016). "Combat Models for RTS Games." URL: https://arxiv.org/pdf/1605.05305

## The Mathematical Models

**Linear Law** (unaimed combat):
- dA/dt = -β·A·B, dB/dt = -α·A·B
- Attrition proportional to product of forces
- Combat power proportional to force size linearly

**Square Law** (aimed combat):
- dA/dt = -β·B, dB/dt = -α·A
- Each unit engages independently
- Combat power proportional to the SQUARE of force size
- **Key insight:** To overcome 2:1 numerical disadvantage, need 4x firepower per unit

## Application to StarCraft

- Unit strength values learned via maximum likelihood from recorded battles
- Faster than full simulation
- Predicts winner AND remaining army composition

## Relevance to Valhalla

Used conceptually in swarm Layer 2 (pile-on detection). When multiple pieces converge on a contested square, the Lanchester square law predicts that concentrated force has quadratic advantage over distributed force. Two players fighting each other weaken each other quadratically while a lurker (free rider) preserves strength. This drives the pile-on penalty: the two most committed players are penalized, the lurker is rewarded.

Requires adaptation: chess pieces capture discretely, not continuously. But the force-ratio concept applies to evaluating who "controls" a contested region.
