import type { RatingResult } from '../types'

// ─── ELO Rating System ────────────────────────────────────────────────────────

const DEFAULT_K = 32

/**
 * Calculate new ELO ratings after a game.
 * @param playerRating  Current player rating
 * @param opponentRating Opponent rating
 * @param score         1 = win, 0.5 = draw, 0 = loss
 * @param k             K-factor (32 for beginners, 16 for established, 10 for top players)
 */
export function calculateElo(
  playerRating:   number,
  opponentRating: number,
  score:          1 | 0.5 | 0,
  k:              number = DEFAULT_K
): RatingResult {
  const expected     = expectedScore(playerRating, opponentRating)
  const change       = Math.round(k * (score - expected))
  const newRating    = playerRating + change

  return {
    newRating,
    ratingChange: change,
  }
}

/** Expected score for player A against player B. */
export function expectedScore(ratingA: number, ratingB: number): number {
  return 1 / (1 + Math.pow(10, (ratingB - ratingA) / 400))
}

/** Determine K-factor based on rating and games played. */
export function kFactor(rating: number, gamesPlayed: number): number {
  if (gamesPlayed < 30) return 40    // New player
  if (rating < 2400)    return 20    // Established player
  return 10                          // Top player
}

/** Convert a win probability to approximate ELO difference. */
export function winProbToEloDiff(prob: number): number {
  if (prob <= 0) return -Infinity
  if (prob >= 1) return Infinity
  return -400 * Math.log10(1 / prob - 1)
}

/** Convert centipawn evaluation to approximate win probability. */
export function cpToWinProb(cp: number): number {
  return 1 / (1 + Math.pow(10, -cp / 400))
}

/** Convert win probability to WDL (Win/Draw/Loss percentages). */
export function winProbToWdl(prob: number): { win: number; draw: number; loss: number } {
  const win  = Math.max(0, Math.min(1, prob))
  const loss = Math.max(0, Math.min(1, 1 - prob))
  const draw = Math.max(0, 1 - win - loss)
  return {
    win:  Math.round(win  * 100),
    draw: Math.round(draw * 100),
    loss: Math.round(loss * 100),
  }
}
