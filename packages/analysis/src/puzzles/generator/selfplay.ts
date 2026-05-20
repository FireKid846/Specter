// ─── Self-Play Game Generator ─────────────────────────────────────────────────
// Drives Specter to play against itself to generate positions for puzzle scanning.

export interface SelfPlayConfig {
  games:     number       // Number of games to generate
  timePerMove: number     // ms per move
  minPly:    number       // Ignore positions before this ply (avoid openings)
  maxPly:    number       // Ignore positions after this ply (avoid simple endgames)
}

export const DEFAULT_SELFPLAY_CONFIG: SelfPlayConfig = {
  games:       100,
  timePerMove: 50,
  minPly:      20,
  maxPly:      100,
}

// Self-play drives the engine via the API — actual game generation
// happens server-side via the Cloudflare Worker calling the WASM engine.
// This module defines the config interface used by the API.
export type SelfPlayCallback = (gameIndex: number, totalGames: number) => void
