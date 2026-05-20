// ─── LLM Coach ────────────────────────────────────────────────────────────────
// Online coaching via Cloudflare Workers AI (Llama 3.1 8B).
// Called from the API — this module is the client interface.

export interface CoachRequest {
  fen:         string
  playedMove:  string
  bestMove:    string
  cpLoss:      number
  annotation:  string
  pv?:         string[]   // Principal variation from engine
  gamePhase?:  'opening' | 'middlegame' | 'endgame'
}

export interface CoachResponse {
  text:     string         // Natural language coaching message
  audio?:   string         // Base64 audio from TTS (if requested)
}

/** Request an AI-generated coaching message from the Specter API. */
export async function requestCoaching(
  req:    CoachRequest,
  apiUrl: string,
  authToken?: string
): Promise<CoachResponse> {
  const res = await fetch(`${apiUrl}/coach/text`, {
    method:  'POST',
    headers: {
      'Content-Type': 'application/json',
      ...(authToken ? { Authorization: `Bearer ${authToken}` } : {}),
    },
    body: JSON.stringify(req),
  })

  if (!res.ok) {
    throw new Error(`Coach API error: ${res.status}`)
  }

  return res.json()
}
