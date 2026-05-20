// ─── Puzzle Scanner ──────────────────────────────────────────────────────────
// Scans game positions for tactical opportunities worth turning into puzzles.

export interface ScanResult {
  fen:          string
  ply:          number
  evalBefore:   number
  evalAfter:    number
  bestMove:     string
  solution:     string[]   // Full solution line
  isTactical:   boolean
}

/** A position is "tactical" if there's a significant eval jump via forced moves. */
export function isTacticalPosition(
  evalBefore:  number,
  evalAfter:   number,
  threshold  = 150
): boolean {
  return Math.abs(evalAfter - evalBefore) >= threshold
}

/** Score how "puzzle-worthy" a position is (higher = better puzzle). */
export function puzzleScore(scan: ScanResult): number {
  const swing     = Math.abs(scan.evalAfter - scan.evalBefore)
  const solutionLen = scan.solution.length
  return swing * (solutionLen <= 5 ? 1.5 : 1.0)
}
