// ─── Opening Trainer ──────────────────────────────────────────────────────────

import { lookupByFen } from '@specter/chess-data'

export interface OpeningTrainerSession {
  targetOpening: string     // ECO code or name to practice
  moves:         string[]   // Moves played so far (UCI)
  currentFen:    string
  isComplete:    boolean
  score:         number     // 0-100
  errors:        number
}

export interface TrainerFeedback {
  correct:       boolean
  message:       string
  hint?:         string
  openingName?:  string
}

/** Check if the user's move matches the expected opening move. */
export function checkOpeningMove(
  playedMove:    string,
  expectedMove:  string,
  currentFen:    string
): TrainerFeedback {
  const opening = lookupByFen(currentFen)

  if (playedMove === expectedMove) {
    return {
      correct:      true,
      message:      '✓ Correct!',
      openingName:  opening?.name,
    }
  }

  return {
    correct: false,
    message: 'Not quite — that deviates from the main line.',
    hint:    `The correct move begins with ${expectedMove.slice(0, 2)}.`,
  }
}

/** Calculate session score based on accuracy and speed. */
export function calcSessionScore(
  totalMoves:   number,
  correctMoves: number,
  timeMs:       number
): number {
  const accuracy    = correctMoves / Math.max(1, totalMoves)
  const timeBonus   = Math.max(0, 1 - timeMs / 60000)   // Bonus for being fast
  return Math.round((accuracy * 0.8 + timeBonus * 0.2) * 100)
}
