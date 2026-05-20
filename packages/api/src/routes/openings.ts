import type { Env } from '../index'
import { json, error } from '../utils/response'
import { lookupByFen, searchByName, byCategory } from '@specter/chess-data'

export async function handleOpenings(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url)

  if (url.pathname === '/openings/identify' && request.method === 'POST') {
    const body = await request.json<{ fen: string }>().catch(() => null)
    if (!body?.fen) return error('Missing fen', 400)
    const entry = lookupByFen(body.fen)
    return json(entry ?? { name: 'Unknown Opening', eco: null })
  }

  if (url.pathname === '/openings/search') {
    const q     = url.searchParams.get('q') ?? ''
    const limit = parseInt(url.searchParams.get('limit') ?? '10')
    return json(searchByName(q, limit))
  }

  if (url.pathname.startsWith('/openings/category/')) {
    const letter = url.pathname.split('/').pop() ?? 'A'
    return json(byCategory(letter))
  }

  return error('Unknown openings endpoint', 404)
}
