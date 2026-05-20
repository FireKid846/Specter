import type { Env } from '../index'

export interface RateLimitResult {
  allowed:    boolean
  remaining:  number
  resetAt:    number
}

const RATE_LIMIT = 100  // requests per minute

export async function rateLimit(request: Request, env: Env): Promise<RateLimitResult> {
  // Use CF's built-in IP for rate limiting key
  const ip  = request.headers.get('CF-Connecting-IP') ?? 'unknown'
  const key = `ratelimit:${ip}:${Math.floor(Date.now() / 60000)}`

  try {
    const current = parseInt(await env.PUZZLES_KV.get(key) ?? '0')
    if (current >= RATE_LIMIT) {
      return { allowed: false, remaining: 0, resetAt: Math.ceil(Date.now() / 60000) * 60000 }
    }
    await env.PUZZLES_KV.put(key, String(current + 1), { expirationTtl: 120 })
    return { allowed: true, remaining: RATE_LIMIT - current - 1, resetAt: 0 }
  } catch {
    return { allowed: true, remaining: RATE_LIMIT, resetAt: 0 }
  }
}
