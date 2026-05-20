// ─── Coach Templates ──────────────────────────────────────────────────────────
// Pre-written coaching messages for offline use (no LLM required).
// Used when voice coach is offline — Web Speech API reads these.

import type { MoveAnnotation } from '@specter/chess-data'

export interface CoachMessage {
  text:     string
  priority: 'praise' | 'warn' | 'encourage' | 'explain'
}

const BLUNDER_MESSAGES = [
  "That move drops a piece — always check for opponent's responses before moving.",
  "Careful! That blunder gives away material. Ask yourself: can my opponent capture something?",
  "Big mistake there. Check all of your opponent's threats before committing to a move.",
]

const BRILLIANT_MESSAGES = [
  "Brilliant move! That's exactly what a strong player would find.",
  "Wow — that's the only move that keeps the advantage. Excellent calculation!",
  "Brilliant! You found a move that even many strong players would miss.",
]

const GREAT_MESSAGES = [
  "Great move! Very close to what the engine would play.",
  "Excellent choice — that's strong chess.",
  "Well played! That's a very good move.",
]

const INACCURACY_MESSAGES = [
  "That's not the worst, but there was a better option. Try to look one move deeper.",
  "Slight inaccuracy — you could improve with a bit more calculation.",
  "Good idea, but the execution could be sharper.",
]

const MISTAKE_MESSAGES = [
  "That was a mistake — it gives your opponent an advantage. Watch out for that pattern.",
  "Mistake! Try to identify what went wrong before continuing.",
  "That move weakens your position significantly.",
]

const MISS_MESSAGES = [
  "You missed a winning tactic! Always scan the board for forks, pins, and discovered attacks.",
  "There was a stronger move available — look for forcing moves first.",
  "Missed opportunity! Check for checks, captures, and threats before quiet moves.",
]

const ENCOURAGEMENT = [
  "You're playing well — keep it up!",
  "Good game so far. Stay focused.",
  "Nice resilience — keep fighting!",
]

export function getCoachMessage(annotation: MoveAnnotation): CoachMessage {
  switch (annotation) {
    case 'brilliant': return pick(BRILLIANT_MESSAGES, 'praise')
    case 'great':     return pick(GREAT_MESSAGES,     'praise')
    case 'best':      return pick(GREAT_MESSAGES,     'praise')
    case 'inaccuracy':return pick(INACCURACY_MESSAGES,'warn')
    case 'mistake':   return pick(MISTAKE_MESSAGES,   'warn')
    case 'blunder':   return pick(BLUNDER_MESSAGES,   'warn')
    case 'miss':      return pick(MISS_MESSAGES,      'warn')
    default:          return pick(ENCOURAGEMENT,      'encourage')
  }
}

export function getOpeningMessage(openingName: string): CoachMessage {
  return {
    text:     `You're playing the ${openingName}. Know its key ideas!`,
    priority: 'explain',
  }
}

export function getEndgameMessage(material: string): CoachMessage {
  return {
    text:     `Endgame reached — ${material}. Activate your king and push passed pawns!`,
    priority: 'explain',
  }
}

function pick(arr: string[], priority: CoachMessage['priority']): CoachMessage {
  const idx = Math.floor(Math.random() * arr.length)
  return { text: arr[idx], priority }
}
