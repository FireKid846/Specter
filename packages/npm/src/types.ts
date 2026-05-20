export interface SpectorConfig {
  elo?:         number    // ELO level (100–3200)
  level?:       string    // Named level (Beginner, Expert, etc.)
  depth?:       number    // Fixed search depth
  timeMs?:      number    // Think time in ms
  style?:       'aggressive' | 'solid' | 'tactical' | 'tricky' | 'chaotic' | 'balanced'
  blunderRate?: number    // 0–1
  openingBook?: boolean
}

export interface MoveResult {
  bestMove:  string    // UCI notation e.g. "e2e4"
  eval:      number    // Centipawns
  depth:     number
  pv:        string[]  // Principal variation
  nodes:     number
  timeMs:    number
}

export type GameState = 'playing' | 'check' | 'checkmate' | 'stalemate' | 'repetition' | 'fifty-move'
