import type { Game, Move } from '../types'

// ─── PGN Exporter ─────────────────────────────────────────────────────────────

export function exportPgn(game: Game): string {
  const lines: string[] = []

  // Headers
  const headerOrder = ['Event','Site','Date','Round','White','Black','Result',
                       'WhiteElo','BlackElo','ECO','Opening','TimeControl']

  for (const key of headerOrder) {
    const lk = key.toLowerCase()
    if (game.headers[lk] !== undefined) {
      lines.push(`[${key} "${game.headers[lk]}"]`)
    }
  }

  // Extra headers
  for (const [k, v] of Object.entries(game.headers)) {
    const canonical = headerOrder.map(h => h.toLowerCase())
    if (!canonical.includes(k)) {
      const display = k.charAt(0).toUpperCase() + k.slice(1)
      lines.push(`[${display} "${v}"]`)
    }
  }

  lines.push('')

  // Moves
  const moveParts: string[] = []
  let moveNumber = 1
  let isWhite = true

  for (const mv of game.moves) {
    if (isWhite) moveParts.push(`${moveNumber}.`)

    let token = mv.san || mv.uci
    if (mv.annotation) token += annotationSymbol(mv.annotation)
    moveParts.push(token)

    if (mv.comment) moveParts.push(`{ ${mv.comment} }`)

    if (!isWhite) moveNumber++
    isWhite = !isWhite
  }

  const result = (game.headers.result as string) ?? '*'
  moveParts.push(result)

  // Wrap at 80 chars
  const moveText = wrapAt80(moveParts.join(' '))
  lines.push(moveText)

  return lines.join('\n')
}

function annotationSymbol(ann: string): string {
  switch (ann) {
    case 'brilliant':  return '!!'
    case 'great':      return '!'
    case 'inaccuracy': return '?!'
    case 'mistake':    return '?'
    case 'blunder':    return '??'
    default:           return ''
  }
}

function wrapAt80(text: string): string {
  const words = text.split(' ')
  const lines: string[] = []
  let current = ''

  for (const word of words) {
    if (current.length + word.length + 1 > 80 && current) {
      lines.push(current)
      current = word
    } else {
      current = current ? `${current} ${word}` : word
    }
  }
  if (current) lines.push(current)
  return lines.join('\n')
}
