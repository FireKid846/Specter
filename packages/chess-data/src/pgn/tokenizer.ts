/// PGN Tokenizer — breaks raw PGN text into tokens.

export type TokenType =
  | 'header_key'
  | 'header_value'
  | 'move_number'
  | 'move'
  | 'annotation'    // !, ?, !!, ??, !?, ?!
  | 'nag'           // $1, $2, etc
  | 'comment'       // { ... }
  | 'result'        // 1-0, 0-1, 1/2-1/2, *
  | 'variation_start'
  | 'variation_end'

export interface Token {
  type:  TokenType
  value: string
}

export function tokenize(pgn: string): Token[] {
  const tokens: Token[] = []
  let i = 0

  while (i < pgn.length) {
    // Skip whitespace
    if (/\s/.test(pgn[i])) { i++; continue }

    // Header tag [Key "Value"]
    if (pgn[i] === '[') {
      i++ // skip [
      const keyEnd = pgn.indexOf(' ', i)
      tokens.push({ type: 'header_key', value: pgn.slice(i, keyEnd) })
      i = keyEnd + 1
      // Skip opening quote
      if (pgn[i] === '"') i++
      const valEnd = pgn.indexOf('"', i)
      tokens.push({ type: 'header_value', value: pgn.slice(i, valEnd) })
      i = valEnd + 2 // skip closing quote and ]
      continue
    }

    // Comment { ... }
    if (pgn[i] === '{') {
      const end = pgn.indexOf('}', i)
      tokens.push({ type: 'comment', value: pgn.slice(i + 1, end).trim() })
      i = end + 1
      continue
    }

    // Variation ( ... )
    if (pgn[i] === '(') { tokens.push({ type: 'variation_start', value: '(' }); i++; continue }
    if (pgn[i] === ')') { tokens.push({ type: 'variation_end',   value: ')' }); i++; continue }

    // NAG $N
    if (pgn[i] === '$') {
      let j = i + 1
      while (j < pgn.length && /\d/.test(pgn[j])) j++
      tokens.push({ type: 'nag', value: pgn.slice(i, j) })
      i = j
      continue
    }

    // Annotation symbols
    if ('!?'.includes(pgn[i])) {
      let j = i
      while (j < pgn.length && '!?'.includes(pgn[j])) j++
      tokens.push({ type: 'annotation', value: pgn.slice(i, j) })
      i = j
      continue
    }

    // Read a word (move number, move, or result)
    let j = i
    while (j < pgn.length && !/[\s\{\}\(\)\[\]]/.test(pgn[j])) j++
    const word = pgn.slice(i, j)
    i = j

    if (!word) continue

    // Result
    if (['1-0', '0-1', '1/2-1/2', '*'].includes(word)) {
      tokens.push({ type: 'result', value: word })
      continue
    }

    // Move number (e.g. "1." or "1...")
    if (/^\d+\.+$/.test(word)) {
      tokens.push({ type: 'move_number', value: word })
      continue
    }

    // Move
    tokens.push({ type: 'move', value: word })
  }

  return tokens
}
