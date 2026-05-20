import type { MoveAnnotation } from '@specter/chess-data'

// ─── Move Classification ──────────────────────────────────────────────────────
// Classifies each move in a game based on centipawn loss vs engine's best.

export interface ClassificationThresholds {
  brilliant:  number   // Rarely matches engine, hard to find
  great:      number   // Very close to engine top choice
  excellent:  number   // Within X cp of best
  good:       number
  inaccuracy: number
  mistake:    number
  // blunder: anything worse than mistake threshold
}

export const DEFAULT_THRESHOLDS: ClassificationThresholds = {
  brilliant:  0,     // Special — detected by engine uniqueness check
  great:      10,    // Within 10cp
  excellent:  25,    // Within 25cp
  good:       50,    // Within 50cp
  inaccuracy: 100,   // Within 100cp
  mistake:    200,   // Within 200cp
  // > 200cp = blunder
}

export interface MoveClassification {
  annotation:    MoveAnnotation
  cpLoss:        number    // Centipawn loss vs engine best
  bestMove:      string    // Engine's suggested move (UCI)
  bestEval:      number    // Eval after best move
  playedEval:    number    // Eval after played move
  explanation?:  string    // Human-readable reason
}

/**
 * Classify a single move.
 * @param bestEval    Evaluation of the position after the engine's best move
 * @param playedEval  Evaluation of the position after the played move
 * @param bestMove    Engine's best move (UCI)
 * @param playedMove  The move that was actually played (UCI)
 * @param isBrilliant Whether the engine detected this as a brilliant move
 */
export function classifyMove(
  bestEval:    number,
  playedEval:  number,
  bestMove:    string,
  playedMove:  string,
  isBrilliant = false,
  thresholds   = DEFAULT_THRESHOLDS
): MoveClassification {
  // Centipawn loss: how much worse was the played move vs best move?
  // Both evals from the perspective of the player who moved.
  const cpLoss = Math.max(0, bestEval - playedEval)

  let annotation: MoveAnnotation

  if (playedMove === bestMove) {
    annotation = 'best'
  } else if (isBrilliant && cpLoss <= 10) {
    annotation = 'brilliant'
  } else if (cpLoss <= thresholds.great) {
    annotation = 'great'
  } else if (cpLoss <= thresholds.excellent) {
    annotation = 'excellent'
  } else if (cpLoss <= thresholds.good) {
    annotation = 'good'
  } else if (cpLoss <= thresholds.inaccuracy) {
    annotation = 'inaccuracy'
  } else if (cpLoss <= thresholds.mistake) {
    annotation = 'mistake'
  } else {
    annotation = 'blunder'
  }

  return {
    annotation,
    cpLoss,
    bestMove,
    bestEval,
    playedEval,
    explanation: buildExplanation(annotation, cpLoss, bestMove, playedMove),
  }
}

/** Check if a move is "brilliant" — only move that maintains advantage. */
export function detectBrilliant(
  bestEval:         number,
  playedEval:       number,
  secondBestEval:   number,
  cpLoss:           number
): boolean {
  // Brilliant if:
  // 1. Very close to best (within 10cp)
  // 2. Second best move is significantly worse (>50cp difference from best)
  // 3. The move is not a simple capture
  const isUnique  = bestEval - secondBestEval > 50
  const isClose   = cpLoss <= 10
  return isUnique && isClose
}

/** Detect a "miss" — player missed a winning tactic that was available. */
export function detectMiss(
  positionEval:     number,   // Eval before move
  playedEval:       number,   // Eval after played move
  tacticalEval:     number,   // Eval if the tactic was played
  threshold = 200
): boolean {
  // A miss: there was a winning tactic available, but player didn't play it
  return tacticalEval - playedEval > threshold && tacticalEval > 200
}

function buildExplanation(
  annotation: MoveAnnotation,
  cpLoss:     number,
  bestMove:   string,
  playedMove: string
): string {
  switch (annotation) {
    case 'brilliant':
      return `Brilliant move! The only way to maintain the advantage.`
    case 'great':
      return `Great move — very close to engine's top choice (${cpLoss}cp from best).`
    case 'best':
      return `Best move — matches engine's top choice.`
    case 'excellent':
      return `Excellent move — only ${cpLoss} centipawns from best.`
    case 'good':
      return `Good move — ${cpLoss} centipawns from best.`
    case 'inaccuracy':
      return `Inaccuracy — lost ${cpLoss} centipawns. Consider ${bestMove}.`
    case 'mistake':
      return `Mistake — lost ${cpLoss} centipawns. The better move was ${bestMove}.`
    case 'blunder':
      return `Blunder — lost ${cpLoss} centipawns! ${bestMove} was much stronger.`
    case 'miss':
      return `Missed a winning tactic! ${bestMove} would have been decisive.`
    default:
      return ''
  }
}
