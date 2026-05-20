# Variants

## Standard Chess
Full FIDE rules. Default variant.

## Chess960 (Fischer Random)
Random starting position for back-rank pieces.
Castling rights encoded differently — king always moves to g/c file.

## Four-Player Chess
14×14 board. 4 separate piece sets.
Search uses multi-player minimax (paranoid algorithm).
Alliance mode supported.

## Three-Check
Win by delivering 3 checks. Check counter tracked per color.
Eval modified to prioritize check-giving moves.

## Horde Chess
White has 36 pawns. Black has standard pieces.
White wins by promoting and queening. Black wins by capturing all pawns.

## Atomic Chess
Captures cause explosions — all pieces on adjacent squares are destroyed.
Special movegen: kings can be adjacent (explosion immunity).
King captures are illegal (would destroy both kings).
