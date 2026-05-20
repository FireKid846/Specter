import { loadWasm } from './wasm-loader'
import type { SpectorConfig, MoveResult, GameState } from './types'

export class SpectorEngine {
  private engine: any = null

  /** Initialize the engine. Must be called before any other method. */
  async init(): Promise<void> {
    const Ctor  = await loadWasm()
    this.engine = new Ctor()
  }

  /** Set position from FEN string. */
  setPosition(fen: string): void {
    this.ensureReady()
    this.engine.setPosition(fen)
  }

  /** Apply a UCI move (e.g. "e2e4") to the current position. */
  makeMove(uci: string): boolean {
    this.ensureReady()
    return this.engine.makeMove(uci)
  }

  /** Get the best move for the current position. */
  async getBestMove(config: SpectorConfig = {}): Promise<MoveResult> {
    this.ensureReady()
    const timeMs = config.timeMs ?? 100
    const depth  = config.depth ?? 0
    if (config.elo) this.engine.setElo(config.elo)
    const uci = this.engine.getBestMove(timeMs, depth)
    return {
      bestMove: uci,
      eval:     this.engine.evaluate(),
      depth:    depth || 10,
      pv:       [uci],
      nodes:    0,
      timeMs,
    }
  }

  /** Get the current FEN. */
  getFen(): string {
    this.ensureReady()
    return this.engine.getFen()
  }

  /** Get all legal moves in UCI notation. */
  getLegalMoves(): string[] {
    this.ensureReady()
    return this.engine.getLegalMoves().split(' ').filter(Boolean)
  }

  /** Get the current game state. */
  getGameState(): GameState {
    this.ensureReady()
    return this.engine.getGameState() as GameState
  }

  /** Reset to starting position. */
  reset(): void {
    this.ensureReady()
    this.engine.reset()
  }

  /** Static evaluation of the current position in centipawns. */
  evaluate(): number {
    this.ensureReady()
    return this.engine.evaluate()
  }

  /** Set ELO strength level. */
  setElo(elo: number): void {
    this.ensureReady()
    this.engine.setElo(elo)
  }

  private ensureReady(): void {
    if (!this.engine) throw new Error('SpectorEngine not initialized. Call init() first.')
  }
}
