// ─── Eval Graph ───────────────────────────────────────────────────────────────
// Converts a list of centipawn evaluations into graph-ready data points.

export interface EvalPoint {
  ply:        number       // Half-move number
  eval:       number       // Centipawn evaluation (White perspective, clamped)
  winProb:    number       // Win probability for White (0-1)
  annotation?: string
  moveSan?:   string
}

const CLAMP_LIMIT = 1000   // Clamp eval display to ±1000cp

export function buildEvalGraph(
  evals:       number[],
  annotations: string[] = [],
  moves:       string[]  = []
): EvalPoint[] {
  return evals.map((cp, i) => ({
    ply:        i + 1,
    eval:       Math.max(-CLAMP_LIMIT, Math.min(CLAMP_LIMIT, cp)),
    winProb:    cpToWinProb(cp),
    annotation: annotations[i],
    moveSan:    moves[i],
  }))
}

export function cpToWinProb(cp: number): number {
  return 1 / (1 + Math.pow(10, -cp / 400))
}

/** Find the biggest eval swings in a game (for highlights). */
export function findCriticalMoments(
  points: EvalPoint[],
  threshold = 100
): number[] {
  const critical: number[] = []
  for (let i = 1; i < points.length; i++) {
    const swing = Math.abs(points[i].eval - points[i - 1].eval)
    if (swing >= threshold) critical.push(i)
  }
  return critical
}

/** Find the turning point of the game (where advantage changed hands). */
export function findTurningPoint(points: EvalPoint[]): number | null {
  let prevSign = Math.sign(points[0]?.eval ?? 0)
  for (let i = 1; i < points.length; i++) {
    const sign = Math.sign(points[i].eval)
    if (sign !== 0 && sign !== prevSign && Math.abs(points[i].eval) > 100) {
      return i
    }
    if (sign !== 0) prevSign = sign
  }
  return null
}
