// ─── Puzzle Validator ─────────────────────────────────────────────────────────

export interface ValidationResult {
  correct:      boolean
  partialCredit: number     // 0-100 percentage
  feedback:     string
  nextHint?:    string
}

/** Validate a user's attempt against the correct solution. */
export function validateSolution(
  userMoves:   string[],   // UCI moves played by user
  solution:    string[]    // Correct solution
): ValidationResult {
  if (userMoves.length === 0) {
    return { correct: false, partialCredit: 0, feedback: 'No moves played.' }
  }

  // Check each move
  let correct = 0
  for (let i = 0; i < Math.min(userMoves.length, solution.length); i++) {
    if (userMoves[i] === solution[i]) correct++
    else break
  }

  if (correct === solution.length) {
    return {
      correct:       true,
      partialCredit: 100,
      feedback:      '✓ Correct! Well done.',
    }
  }

  const partial = Math.round((correct / solution.length) * 80)
  const hint    = solution[correct]

  return {
    correct:       false,
    partialCredit: partial,
    feedback:      correct === 0
      ? 'Incorrect. Try a different move.'
      : `Good start! You got ${correct} of ${solution.length} moves right.`,
    nextHint: `Hint: the next move starts with ${hint.slice(0, 2)}.`,
  }
}
