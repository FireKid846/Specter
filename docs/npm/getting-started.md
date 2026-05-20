# Getting Started with specter-engine

```bash
npm install specter-engine
```

## Basic Usage

```typescript
import { createEngine } from 'specter-engine'

const engine = await createEngine()

// Set position from FEN
engine.setPosition('rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1')

// Get best move (100ms think time)
const result = await engine.getBestMove({ timeMs: 100 })
console.log(result.bestMove)  // e.g. "e2e4"
console.log(result.eval)      // centipawn evaluation
console.log(result.pv)        // principal variation

// Apply a move
engine.makeMove('e2e4')

// Check game state
const state = engine.getGameState()
// 'playing' | 'check' | 'checkmate' | 'stalemate' | 'repetition' | 'fifty-move'

// Get all legal moves
const moves = engine.getLegalMoves()
// ['e2e4', 'd2d4', 'g1f3', ...]
```

## Skill Levels

```typescript
// By ELO
const result = await engine.getBestMove({ elo: 1200 })

// By named level
const result = await engine.getBestMove({ level: 'Intermediate' })

// By raw params
const result = await engine.getBestMove({
  depth:       8,
  blunderRate: 0.05,
  style:       'aggressive',
})
```

## Playing Styles

- `aggressive` — Prefers attacking moves and sacrifices
- `solid`      — Positional, avoids risk
- `tactical`   — Seeks combinations and tactics
- `tricky`     — Sets traps, avoids main lines
- `chaotic`    — Unpredictable play
- `balanced`   — Default neutral style
