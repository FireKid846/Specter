import type { Env } from '../index'
import { json, error } from '../utils/response'

export async function handlePuzzles(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url)

  if (url.pathname === '/puzzles/daily') {
    return handleDailyPuzzle(env)
  }
  if (url.pathname === '/puzzles/random') {
    return handleRandomPuzzle(request, env)
  }
  if (url.pathname === '/puzzles/validate' && request.method === 'POST') {
    return handleValidatePuzzle(request, env)
  }
  if (url.pathname === '/puzzles/generate' && request.method === 'POST') {
    return handleGeneratePuzzle(request, env)
  }

  return error('Unknown puzzle endpoint', 404)
}

async function handleDailyPuzzle(env: Env): Promise<Response> {
  const today = new Date().toISOString().split('T')[0]
  const cached = await env.PUZZLES_KV.get(`daily:${today}`)
  if (cached) return json(JSON.parse(cached))

  const puzzle = {
    id:         `daily-${today}`,
    fen:        'r1bqkb1r/pppp1ppp/2n2n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R w KQkq - 4 4',
    moves:      ['f3g5', 'f6e4', 'g5f7'],
    themes:     ['fork', 'mateIn2'],
    difficulty: 'medium',
    rating:     1450,
  }

  await env.PUZZLES_KV.put(`daily:${today}`, JSON.stringify(puzzle), {
    expirationTtl: 86400
  })

  return json(puzzle)
}

async function handleRandomPuzzle(request: Request, env: Env): Promise<Response> {
  const url    = new URL(request.url)
  const rating = parseInt(url.searchParams.get('rating') ?? '1500')
  const theme  = url.searchParams.get('theme') ?? ''

  // In production: query D1 for a puzzle matching criteria
  return json({
    id:         'puzzle-001',
    fen:        '6k1/5ppp/8/8/8/8/5PPP/R5K1 w - - 0 1',
    moves:      ['a1a8'],
    themes:     ['backRankMate', 'mateIn1'],
    difficulty: 'easy',
    rating:     900,
  })
}

async function handleValidatePuzzle(request: Request, env: Env): Promise<Response> {
  const body = await request.json<{ puzzleId: string; moves: string[] }>()
  // TODO: validate against stored solution
  return json({ correct: true, partialCredit: 100, feedback: 'Correct!' })
}

async function handleGeneratePuzzle(request: Request, env: Env): Promise<Response> {
  // Server-side puzzle generation via self-play
  // Long-running — use Durable Objects or Queue in production
  return json({ status: 'queued', jobId: crypto.randomUUID() })
}
