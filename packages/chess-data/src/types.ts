// ─── Core chess types ──────────────────────────────────────────────────────────

export type Color = 'w' | 'b'
export type PieceSymbol = 'p' | 'n' | 'b' | 'r' | 'q' | 'k'
export type Square = string  // e.g. "e4"

export interface Move {
  from:        Square
  to:          Square
  promotion?:  PieceSymbol
  san?:        string    // Standard Algebraic Notation
  uci:         string    // UCI notation e.g. "e2e4"
  fen?:        string    // FEN after this move
  eval?:       number    // Centipawn evaluation
  annotation?: MoveAnnotation
  comment?:    string
  clock?:      number    // Clock time remaining in ms
}

export type MoveAnnotation =
  | 'brilliant'   // !!
  | 'great'       // !
  | 'best'        // engine's top choice
  | 'excellent'   // very close to best
  | 'good'        // fine move
  | 'inaccuracy'  // ?!
  | 'mistake'     // ?
  | 'blunder'     // ??
  | 'miss'        // missed a winning tactic

export interface GameHeader {
  event?:     string
  site?:      string
  date?:      string
  round?:     string
  white?:     string
  black?:     string
  result?:    '1-0' | '0-1' | '1/2-1/2' | '*'
  whiteElo?:  number
  blackElo?:  number
  eco?:       string
  opening?:   string
  variation?: string
  timeControl?: string
  termination?: string
  [key: string]: string | number | undefined
}

export interface Game {
  headers: GameHeader
  moves:   Move[]
  pgn:     string
}

export interface EcoEntry {
  eco:      string   // e.g. "B90"
  name:     string   // e.g. "Sicilian Defense: Najdorf Variation"
  pgn:      string   // moves in PGN format
  fen:      string   // FEN of the position
}

export interface RatingResult {
  newRating:  number
  newRd?:     number  // Glicko rating deviation
  newVol?:    number  // Glicko volatility
  ratingChange: number
}
