# Evaluation

## Score Packing

MG and EG scores are packed into a single i32:
- Low 16 bits: middlegame score
- High 16 bits: endgame score

This allows adding/subtracting scores in one operation.

## Piece Values

| Piece  | MG    | EG    |
|--------|-------|-------|
| Pawn   |  126  |  208  |
| Knight |  781  |  854  |
| Bishop |  825  |  915  |
| Rook   | 1276  | 1380  |
| Queen  | 2538  | 2682  |

Bishop pair bonus: +23 MG, +62 EG

## PST Source

Piece-square tables from PeSTO (Piece-Square Tables Only),
which are among the strongest known hand-crafted PSTs.

## Pawn Structure

- **Passed pawn**: Bonus by rank (large — passed pawns are decisive in endgames)
- **Isolated pawn**: -15 MG, -20 EG
- **Doubled pawn**: -10 MG, -20 EG
- **Backward pawn**: -12 MG, -10 EG
- **Phalanx**: +8 MG, +5 EG
- **Connected**: +7 MG, +8 EG

## King Safety

Attack zone = king square + 8 adjacent squares.
Each attacker type contributes a weighted danger value.
Danger is scaled by the number of attackers (non-linear).
