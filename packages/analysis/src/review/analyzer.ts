import type { Game, Move, MoveAnnotation } from '@specter/chess-data'
import { classifyMove, detectBrilliant, detectMiss } from './classifier'

// ─── Game Analyzer ────────────────────────────────────────────────────────────

export interface AnalyzedMove extends Move {
  annotation:    MoveAnnotation
  cpLoss:        number
  bestMove:      string
  playedEval:    number
  bestEval:      number
  explanation:   string
  isMiss:        boolean
}

export interface GameAnalysis {
  moves:          AnalyzedMove[]
  evalHistory:    number[]      // Eval after each move (White perspective)
  summary:        GameSummary
  whiteAccuracy:  number        // 0-100
  blackAccuracy:  number        // 0-100
}

export interface GameSummary {
  white: PlayerSummary
  black: PlayerSummary
}

export interface PlayerSummary {
  brilliant:   number
  great:       number
  best:        number
  excellent:   number
  good:        number
  inaccuracy:  number
  mistake:     number
  blunder:     number
  miss:        number
  averageCpLoss: number
}

/** Analyze a game given a list of (fen, bestMove, bestEval, playedEval) tuples. */
export function analyzeGame(
  game:         Game,
  engineData:   EngineData[]
): GameAnalysis {
  const analyzedMoves: AnalyzedMove[] = []
  const evalHistory: number[]         = []

  const whiteSummary = emptySummary()
  const blackSummary = emptySummary()

  for (let i = 0; i < engineData.length; i++) {
    const mv   = game.moves[i]
    if (!mv) break

    const data = engineData[i]
    const isWhite = i % 2 === 0

    const classified = classifyMove(
      data.bestEval,
      data.playedEval,
      data.bestMove,
      mv.uci || '',
      data.isBrilliant ?? false
    )

    const isMiss = detectMiss(
      data.positionEval ?? 0,
      data.playedEval,
      data.tacticalEval ?? data.playedEval
    )

    const annotation: MoveAnnotation = isMiss && classified.annotation === 'blunder'
      ? 'miss'
      : classified.annotation

    analyzedMoves.push({
      ...mv,
      annotation,
      cpLoss:      classified.cpLoss,
      bestMove:    classified.bestMove,
      playedEval:  classified.playedEval,
      bestEval:    classified.bestEval,
      explanation: classified.explanation,
      isMiss,
    })

    evalHistory.push(data.playedEval)

    const summary = isWhite ? whiteSummary : blackSummary
    updateSummary(summary, annotation, classified.cpLoss)
  }

  whiteSummary.averageCpLoss = avgCpLoss(analyzedMoves.filter((_, i) => i % 2 === 0))
  blackSummary.averageCpLoss = avgCpLoss(analyzedMoves.filter((_, i) => i % 2 !== 0))

  return {
    moves:         analyzedMoves,
    evalHistory,
    summary:       { white: whiteSummary, black: blackSummary },
    whiteAccuracy: summaryToAccuracy(whiteSummary),
    blackAccuracy: summaryToAccuracy(blackSummary),
  }
}

export interface EngineData {
  fen:           string
  bestMove:      string
  bestEval:      number    // Eval after best move (cp, from mover's perspective)
  playedEval:    number    // Eval after played move
  positionEval?: number    // Eval before move
  tacticalEval?: number    // Eval of best tactical move (for miss detection)
  isBrilliant?:  boolean
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

function emptySummary(): PlayerSummary {
  return {
    brilliant: 0, great: 0, best: 0, excellent: 0, good: 0,
    inaccuracy: 0, mistake: 0, blunder: 0, miss: 0, averageCpLoss: 0
  }
}

function updateSummary(s: PlayerSummary, ann: MoveAnnotation, cpLoss: number) {
  switch (ann) {
    case 'brilliant':  s.brilliant++;  break
    case 'great':      s.great++;      break
    case 'best':       s.best++;       break
    case 'excellent':  s.excellent++;  break
    case 'good':       s.good++;       break
    case 'inaccuracy': s.inaccuracy++; break
    case 'mistake':    s.mistake++;    break
    case 'blunder':    s.blunder++;    break
    case 'miss':       s.miss++;       break
  }
}

function avgCpLoss(moves: AnalyzedMove[]): number {
  if (!moves.length) return 0
  return Math.round(moves.reduce((sum, m) => sum + m.cpLoss, 0) / moves.length)
}

/** Convert a player summary to an accuracy score (0–100). */
function summaryToAccuracy(s: PlayerSummary): number {
  const total = s.brilliant + s.great + s.best + s.excellent +
                s.good + s.inaccuracy + s.mistake + s.blunder + s.miss
  if (!total) return 0

  const weighted =
    s.brilliant  * 100 +
    s.great      *  95 +
    s.best       * 100 +
    s.excellent  *  90 +
    s.good       *  80 +
    s.inaccuracy *  60 +
    s.mistake    *  30 +
    s.blunder    *   0 +
    s.miss       *   0

  return Math.round(weighted / total)
}
