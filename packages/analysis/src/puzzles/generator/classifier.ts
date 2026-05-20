// ─── Puzzle Classifier ────────────────────────────────────────────────────────

export type PuzzleTheme =
  | 'mateIn1' | 'mateIn2' | 'mateIn3' | 'mateIn4'
  | 'fork' | 'pin' | 'skewer' | 'discoveredAttack'
  | 'backRankMate' | 'smotheredMate' | 'queenSacrifice'
  | 'promotion' | 'endgame' | 'middlegame' | 'opening'
  | 'defensiveMove' | 'zugzwang' | 'stalemate' | 'trapping'

export type PuzzleDifficulty = 'beginner' | 'easy' | 'medium' | 'hard' | 'expert'

export interface Puzzle {
  id:           string
  fen:          string          // Starting position
  moves:        string[]        // Solution moves (UCI)
  themes:       PuzzleTheme[]
  difficulty:   PuzzleDifficulty
  rating:       number          // Puzzle ELO rating
  source?:      string          // "self-play" | "game" | "composed"
  gameId?:      string
  ply?:         number          // Ply in source game
}

export function classifyPuzzle(
  fen:      string,
  solution: string[],
  evalBefore: number,
  evalAfter:  number
): Partial<Puzzle> {
  const themes: PuzzleTheme[] = []

  // Mate detection
  if (solution.length === 1 && evalAfter > 29000) themes.push('mateIn1')
  else if (solution.length === 3 && evalAfter > 29000) themes.push('mateIn2')
  else if (solution.length === 5 && evalAfter > 29000) themes.push('mateIn3')
  else if (solution.length === 7 && evalAfter > 29000) themes.push('mateIn4')

  // Promotion
  const solutionStr = solution.join(' ')
  if (/[a-h][27][a-h][18][qrbn]/i.test(solutionStr)) themes.push('promotion')

  // Difficulty based on solution length + eval swing
  const swing = evalAfter - evalBefore
  let difficulty: PuzzleDifficulty = 'medium'
  if (solution.length === 1)  difficulty = 'beginner'
  else if (solution.length === 3 && swing > 500) difficulty = 'easy'
  else if (solution.length <= 5) difficulty = 'medium'
  else if (solution.length <= 9) difficulty = 'hard'
  else difficulty = 'expert'

  // Rating estimate
  const baseRating = {
    beginner: 800,
    easy:     1200,
    medium:   1500,
    hard:     1900,
    expert:   2300,
  }[difficulty]

  const rating = baseRating + Math.floor(Math.random() * 200)

  return { fen, moves: solution, themes, difficulty, rating }
}

export function ratePuzzle(puzzle: Puzzle): number {
  const complexityScore = puzzle.moves.length * 100
  const themeBonus      = puzzle.themes.length * 50
  return Math.min(3000, 800 + complexityScore + themeBonus)
}
