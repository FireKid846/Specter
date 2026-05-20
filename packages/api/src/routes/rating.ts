import type { Env } from '../index'
import { json, error } from '../utils/response'
import { calculateElo, calculateGlicko2 } from '@specter/chess-data'

export async function handleRating(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url)

  if (url.pathname === '/rating/calculate' && request.method === 'POST') {
    const body = await request.json<{
      system:          'elo' | 'glicko2'
      playerRating:    number
      opponentRating:  number
      score:           1 | 0.5 | 0
      playerRd?:       number
      playerVol?:      number
      opponentRd?:     number
    }>().catch(() => null)

    if (!body) return error('Invalid body', 400)

    if (body.system === 'glicko2') {
      const result = calculateGlicko2(
        { rating: body.playerRating, rd: body.playerRd ?? 350, volatility: body.playerVol ?? 0.06 },
        [{ opponentRating: body.opponentRating, opponentRd: body.opponentRd ?? 350, score: body.score }]
      )
      return json(result)
    }

    const result = calculateElo(body.playerRating, body.opponentRating, body.score)
    return json(result)
  }

  return error('Unknown rating endpoint', 404)
}
