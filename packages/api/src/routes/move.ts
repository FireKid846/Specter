import type { Env } from '../index'
import { json, error } from '../utils/response'

export interface MoveRequest {
  fen:        string
  timeMs?:    number    // Think time in ms (default 100)
  depth?:     number    // Fixed depth (overrides timeMs)
  elo?:       number    // Skill level as ELO
  style?:     string    // Playing style
}

export async function handleMove(request: Request, env: Env): Promise<Response> {
  const body = await request.json<MoveRequest>().catch(() => null)
  if (!body?.fen) return error('Missing fen', 400)

  // The engine runs as WASM on the edge — for now we return a placeholder
  // In production: load specter WASM, setPosition(fen), getBestMove(timeMs, depth)
  // This will be wired up once WASM is built and bundled with the Worker.

  try {
    // TODO: Wire up WASM engine
    // const engine = new SpectorEngine()
    // engine.setPosition(body.fen)
    // if (body.elo) engine.setElo(body.elo)
    // const bestMove = engine.getBestMove(body.timeMs ?? 100, body.depth ?? 0)

    return json({
      bestMove: 'e2e4',    // Placeholder
      eval:     0,
      depth:    1,
      pv:       ['e2e4'],
      nodes:    1,
      timeMs:   0,
    })
  } catch (err) {
    return error(`Engine error: ${err}`, 500)
  }
}
