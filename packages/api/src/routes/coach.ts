import type { Env } from '../index'
import { json, error } from '../utils/response'
import { getCoachMessage } from '@specter/analysis'

export async function handleCoach(request: Request, env: Env): Promise<Response> {
  const url = new URL(request.url)

  if (url.pathname === '/coach/text' && request.method === 'POST') {
    return handleCoachText(request, env)
  }
  if (url.pathname === '/coach/voice' && request.method === 'POST') {
    return handleCoachVoice(request, env)
  }

  return error('Unknown coach endpoint', 404)
}

async function handleCoachText(request: Request, env: Env): Promise<Response> {
  const body = await request.json<{
    fen:        string
    playedMove: string
    bestMove:   string
    cpLoss:     number
    annotation: string
    pv?:        string[]
  }>().catch(() => null)

  if (!body) return error('Invalid request body', 400)

  // Use Cloudflare Workers AI (Llama) for natural language coaching
  const prompt = buildCoachPrompt(body)

  try {
    const aiResponse = await env.AI.run('@cf/meta/llama-3.1-8b-instruct', {
      messages: [
        {
          role:    'system',
          content: `You are a friendly chess coach named Specter. Give concise, helpful advice 
                    in 1-2 sentences. Be encouraging but honest. Focus on the specific move 
                    and what the player should learn from it.`
        },
        { role: 'user', content: prompt }
      ],
      max_tokens: 100,
    })

    const text = aiResponse.response ?? getCoachMessage(body.annotation as any).text
    return json({ text })
  } catch {
    // Fallback to template message
    const msg = getCoachMessage(body.annotation as any)
    return json({ text: msg.text })
  }
}

async function handleCoachVoice(request: Request, env: Env): Promise<Response> {
  const body = await request.json<{ text: string }>().catch(() => null)
  if (!body?.text) return error('Missing text', 400)

  try {
    // Cloudflare Workers AI TTS
    const audio = await env.AI.run('@cf/myshell-ai/melotts', {
      prompt: body.text,
    })

    return new Response(audio, {
      headers: { 'Content-Type': 'audio/mpeg' }
    })
  } catch {
    return error('TTS unavailable', 503)
  }
}

function buildCoachPrompt(body: {
  playedMove: string
  bestMove:   string
  cpLoss:     number
  annotation: string
}): string {
  return `The player just played ${body.playedMove}. 
The engine's best move was ${body.bestMove}. 
This was classified as a ${body.annotation} (${body.cpLoss} centipawn loss).
Give a short coaching tip about this specific move.`
}
