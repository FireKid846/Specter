import type { RatingResult } from '../types'

// ─── Glicko-2 Rating System ───────────────────────────────────────────────────
// More accurate than ELO — accounts for rating reliability (RD) and volatility.

const TAU      = 0.5    // System constant — constrains volatility change
const EPSILON  = 0.000001
const Q        = Math.log(10) / 400  // Glicko-1 constant

export interface Glicko2Player {
  rating:     number  // Glicko-2 rating (typically 1500 initial)
  rd:         number  // Rating deviation (typically 350 initial)
  volatility: number  // Volatility (typically 0.06 initial)
}

export interface GameResult {
  opponentRating:     number
  opponentRd:         number
  score:              1 | 0.5 | 0
}

/**
 * Calculate new Glicko-2 rating after a rating period.
 * @param player   Current player stats
 * @param results  Array of games played in the rating period
 */
export function calculateGlicko2(
  player:  Glicko2Player,
  results: GameResult[]
): RatingResult & { newRd: number; newVol: number } {
  // Step 1: Convert to Glicko-2 scale
  const mu  = (player.rating - 1500) / 173.7178
  const phi = player.rd / 173.7178

  if (results.length === 0) {
    // No games — increase RD
    const newPhi = Math.sqrt(phi ** 2 + player.volatility ** 2)
    return {
      newRating:    Math.round(173.7178 * mu + 1500),
      newRd:        Math.round(173.7178 * newPhi),
      newVol:       player.volatility,
      ratingChange: 0,
    }
  }

  // Step 2: Compute v (estimated variance)
  let v = 0
  for (const r of results) {
    const muJ  = (r.opponentRating - 1500) / 173.7178
    const phiJ = r.opponentRd / 173.7178
    const gJ   = gFunc(phiJ)
    const eJ   = eFunc(mu, muJ, phiJ)
    v += gJ ** 2 * eJ * (1 - eJ)
  }
  v = 1 / v

  // Step 3: Compute delta (estimated improvement)
  let delta = 0
  for (const r of results) {
    const muJ  = (r.opponentRating - 1500) / 173.7178
    const phiJ = r.opponentRd / 173.7178
    const gJ   = gFunc(phiJ)
    const eJ   = eFunc(mu, muJ, phiJ)
    delta += gJ * (r.score - eJ)
  }
  delta *= v

  // Step 4: Update volatility using Illinois algorithm
  const newVol = updateVolatility(phi, v, delta, player.volatility)

  // Step 5: Update RD
  const phiStar = Math.sqrt(phi ** 2 + newVol ** 2)
  const newPhi  = 1 / Math.sqrt(1 / phiStar ** 2 + 1 / v)

  // Step 6: Update rating
  let muNew = mu
  for (const r of results) {
    const muJ  = (r.opponentRating - 1500) / 173.7178
    const phiJ = r.opponentRd / 173.7178
    const gJ   = gFunc(phiJ)
    const eJ   = eFunc(mu, muJ, phiJ)
    muNew += newPhi ** 2 * gJ * (r.score - eJ)
  }

  const newRating = Math.round(173.7178 * muNew + 1500)
  const newRd     = Math.round(173.7178 * newPhi)

  return {
    newRating,
    newRd,
    newVol,
    ratingChange: newRating - player.rating,
  }
}

function gFunc(phi: number): number {
  return 1 / Math.sqrt(1 + 3 * phi ** 2 / Math.PI ** 2)
}

function eFunc(mu: number, muJ: number, phiJ: number): number {
  return 1 / (1 + Math.exp(-gFunc(phiJ) * (mu - muJ)))
}

function updateVolatility(
  phi: number,
  v:   number,
  delta: number,
  sigma: number
): number {
  const a  = Math.log(sigma ** 2)
  const f  = (x: number) => {
    const ex = Math.exp(x)
    const d  = phi ** 2 + v + ex
    return ex * (delta ** 2 - phi ** 2 - v - ex) / (2 * d ** 2)
         - (x - a) / TAU ** 2
  }

  let A = a
  let B: number

  if (delta ** 2 > phi ** 2 + v) {
    B = Math.log(delta ** 2 - phi ** 2 - v)
  } else {
    let k = 1
    while (f(a - k * TAU) < 0) k++
    B = a - k * TAU
  }

  let fA = f(A)
  let fB = f(B)

  while (Math.abs(B - A) > EPSILON) {
    const C  = A + (A - B) * fA / (fB - fA)
    const fC = f(C)
    if (fC * fB <= 0) { A = B; fA = fB }
    else              { fA /= 2 }
    B  = C
    fB = fC
  }

  return Math.exp(A / 2)
}

/** Create a new player with default Glicko-2 values. */
export function newPlayer(rating = 1500): Glicko2Player {
  return { rating, rd: 350, volatility: 0.06 }
}
