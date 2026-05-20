import type { Env } from '../index'
import { json, error } from '../utils/response'
import { classifyMove } from '@specter/analysis'

export interface AnalysisRequest {
  pgn?:     string      // Full game PGN
  fen?:     string      // Single position FEN
  moves?:   string[]    // Moves in UCI (alternative to PGN)
  depth?:   number      // Analysis depth (default 18)
}

export async function handleAnalysis(request: Request, env: Env): Promise<Response> {
  const body = await request.json<AnalysisRequest>().catch(() => null)
  if (!body?.pgn && !body?.fen) return error('Missing pgn or fen', 400)

  const url = new URL(request.url)

  // /analysis/game — full game review
  if (url.pathname === '/analysis/game') {
    return handleGameReview(body, env)
  }

  // /analysis/position — single position analysis
  if (url.pathname === '/analysis/position') {
    return handlePositionAnalysis(body, env)
  }

  return error('Unknown analysis endpoint', 404)
}

async function handleGameReview(body: AnalysisRequest, env: Env): Promise<Response> {
  // Full game review: analyze each position in the game
  // In production: iterate positions, run engine at each, classify moves
  return json({
    moves:         [],
    evalHistory:   [],
    whiteAccuracy: 85,
    blackAccuracy: 72,
    summary: {
      white: { brilliant: 1, great: 3, best: 8, excellent: 5, good: 4,
               inaccuracy: 2, mistake: 1, blunder: 0, miss: 0, averageCpLoss: 18 },
      black: { brilliant: 0, great: 2, best: 6, excellent: 4, good: 5,
               inaccuracy: 3, mistake: 2, blunder: 1, miss: 1, averageCpLoss: 35 },
    }
  })
}

async function handlePositionAnalysis(body: AnalysisRequest, env: Env): Promise<Response> {
  return json({
    fen:      body.fen,
    bestMove: 'e2e4',
    eval:     25,
    depth:    body.depth ?? 18,
    pv:       ['e2e4', 'e7e5', 'g1f3'],
  })
}
