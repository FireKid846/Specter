# Search

## Iterative Deepening

Specter searches increasing depths (1, 2, 3, ...) until the time or depth
limit is reached. This ensures a move is always available if time runs out.

## Aspiration Windows

At depth ≥ 4, search starts with a narrow window `[score-25, score+25]`.
If the search fails low or high, the window is widened and the search repeats.

## Move Ordering

Good move ordering is critical — the better moves are tried first,
the more alpha-beta prunes the search tree.

Order (highest priority first):
1. TT move (best move from transposition table)
2. Captures (MVV-LVA: most valuable victim, least valuable aggressor)
3. Promotions
4. Killer moves (quiet moves that caused beta cutoffs at this ply)
5. Butterfly history (accumulated move scores from previous searches)

## LMR Table

```
reduction = 0.75 + ln(depth) × ln(moveNumber) / 2.25
```

## NMP Formula

```
R = 3 + depth/4 + min(3, (static_eval - beta) / 200)
```
