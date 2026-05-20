import { tokenize } from './tokenizer'
import type { Game, GameHeader, Move, MoveAnnotation } from '../types'

// ─── Annotation mapping ───────────────────────────────────────────────────────

const ANNOTATION_MAP: Record<string, MoveAnnotation> = {
  '!!': 'brilliant',
  '!':  'great',
  '!?': 'excellent',
  '?!': 'inaccuracy',
  '?':  'mistake',
  '??': 'blunder',
}

const NAG_MAP: Record<string, MoveAnnotation> = {
  '$1':  'great',
  '$2':  'mistake',
  '$3':  'brilliant',
  '$4':  'blunder',
  '$5':  'inaccuracy',
  '$6':  'inaccuracy',
}

// ─── Parser ───────────────────────────────────────────────────────────────────

export function parsePgn(pgn: string): Game {
  const tokens = tokenize(pgn)
  const headers: GameHeader = {}
  const moves: Move[] = []

  let i = 0
  let pendingAnnotation: MoveAnnotation | undefined
  let pendingComment: string | undefined

  // Parse headers
  while (i < tokens.length && tokens[i].type === 'header_key') {
    const key   = tokens[i].value
    const value = tokens[i + 1]?.type === 'header_value' ? tokens[i + 1].value : ''
    headers[key.toLowerCase()] = value
    i += 2
  }

  // Parse move text
  while (i < tokens.length) {
    const tok = tokens[i]

    if (tok.type === 'result') { i++; continue }
    if (tok.type === 'move_number') { i++; continue }
    if (tok.type === 'variation_start' || tok.type === 'variation_end') { i++; continue }

    if (tok.type === 'comment') {
      pendingComment = tok.value
      i++
      continue
    }

    if (tok.type === 'annotation') {
      pendingAnnotation = ANNOTATION_MAP[tok.value]
      i++
      continue
    }

    if (tok.type === 'nag') {
      pendingAnnotation = NAG_MAP[tok.value] ?? pendingAnnotation
      i++
      continue
    }

    if (tok.type === 'move') {
      const san = tok.value
      moves.push({
        from:        '',   // SAN → from/to resolved by engine
        to:          '',
        uci:         '',
        san,
        annotation:  pendingAnnotation,
        comment:     pendingComment,
      })
      pendingAnnotation = undefined
      pendingComment    = undefined
      i++
      continue
    }

    i++
  }

  return { headers, moves, pgn }
}

// ─── Multi-game parser ────────────────────────────────────────────────────────

export function parseMultiPgn(pgn: string): Game[] {
  // Split on event headers
  const games = pgn.split(/(?=\[Event )/).filter(g => g.trim())
  return games.map(parsePgn)
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

export function extractFens(game: Game): string[] {
  return game.moves
    .map(m => m.fen)
    .filter((f): f is string => !!f)
}

export function gameResult(game: Game): '1-0' | '0-1' | '1/2-1/2' | '*' {
  return (game.headers.result as any) ?? '*'
}
