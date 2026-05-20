// ─── Opening Quiz ─────────────────────────────────────────────────────────────

import { lookupByFen, byCategory } from '@specter/chess-data'
import type { EcoEntry } from '@specter/chess-data'

export interface QuizQuestion {
  id:          string
  type:        'name-the-opening' | 'play-the-move' | 'identify-mistake'
  fen:         string
  question:    string
  options?:    string[]           // For multiple choice
  answer:      string
  explanation: string
}

/** Generate a "name the opening" quiz question. */
export function generateNameQuestion(entry: EcoEntry): QuizQuestion {
  const wrongOptions = byCategory(entry.eco[0])
    .filter(e => e.eco !== entry.eco)
    .slice(0, 3)
    .map(e => e.name)

  const options = shuffle([entry.name, ...wrongOptions])

  return {
    id:          `name-${entry.eco}`,
    type:        'name-the-opening',
    fen:         entry.fen,
    question:    'What opening is this?',
    options,
    answer:      entry.name,
    explanation: `This is the ${entry.name} (${entry.eco}). ${entry.pgn}`,
  }
}

/** Shuffle an array (Fisher-Yates). */
function shuffle<T>(arr: T[]): T[] {
  const a = [...arr]
  for (let i = a.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [a[i], a[j]] = [a[j], a[i]]
  }
  return a
}

/** Score a quiz attempt. */
export function scoreQuiz(
  questions: QuizQuestion[],
  answers:   string[]
): { score: number; correct: number; total: number } {
  let correct = 0
  for (let i = 0; i < questions.length; i++) {
    if (answers[i] === questions[i].answer) correct++
  }
  return {
    score:   Math.round((correct / questions.length) * 100),
    correct,
    total:   questions.length,
  }
}
