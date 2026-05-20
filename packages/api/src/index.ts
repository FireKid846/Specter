// ─── Specter API — Cloudflare Worker ─────────────────────────────────────────

import { handleMove }     from './routes/move'
import { handleAnalysis } from './routes/analysis'
import { handlePuzzles }  from './routes/puzzles'
import { handleCoach }    from './routes/coach'
import { handleOpenings } from './routes/openings'
import { handleRating }   from './routes/rating'
import { corsHeaders }    from './middleware/cors'
import { rateLimit }      from './middleware/ratelimit'
import { validateRequest } from './utils/validation'

export interface Env {
  AI:         any           // Cloudflare Workers AI binding
  PUZZLES_KV: KVNamespace   // KV for puzzle storage
  DB:         D1Database    // D1 for user data
  ENVIRONMENT: string
}

export default {
  async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
    const url = new URL(request.url)

    // CORS preflight
    if (request.method === 'OPTIONS') {
      return new Response(null, { headers: corsHeaders() })
    }

    // Rate limiting
    const rateLimitResult = await rateLimit(request, env)
    if (!rateLimitResult.allowed) {
      return new Response(JSON.stringify({ error: 'Rate limit exceeded' }), {
        status:  429,
        headers: { 'Content-Type': 'application/json', ...corsHeaders() },
      })
    }

    try {
      let response: Response

      switch (true) {
        case url.pathname === '/move'              && request.method === 'POST':
          response = await handleMove(request, env); break
        case url.pathname.startsWith('/analysis') && request.method === 'POST':
          response = await handleAnalysis(request, env); break
        case url.pathname.startsWith('/puzzles'):
          response = await handlePuzzles(request, env); break
        case url.pathname.startsWith('/coach'):
          response = await handleCoach(request, env); break
        case url.pathname.startsWith('/openings'):
          response = await handleOpenings(request, env); break
        case url.pathname.startsWith('/rating'):
          response = await handleRating(request, env); break
        case url.pathname === '/health':
          response = new Response(JSON.stringify({ status: 'ok', version: '0.1.0' }), {
            headers: { 'Content-Type': 'application/json' },
          }); break
        default:
          response = new Response(JSON.stringify({ error: 'Not found' }), { status: 404 })
      }

      // Add CORS headers to all responses
      const headers = new Headers(response.headers)
      Object.entries(corsHeaders()).forEach(([k, v]) => headers.set(k, v))
      return new Response(response.body, { status: response.status, headers })

    } catch (err) {
      console.error('Specter API error:', err)
      return new Response(JSON.stringify({ error: 'Internal server error' }), {
        status:  500,
        headers: { 'Content-Type': 'application/json', ...corsHeaders() },
      })
    }
  }
}
