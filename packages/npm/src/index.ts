export { SpectorEngine } from './engine'
export type { SpectorConfig, MoveResult, GameState } from './types'

// Convenience factory
import { SpectorEngine } from './engine'
export async function createEngine(): Promise<SpectorEngine> {
  const engine = new SpectorEngine()
  await engine.init()
  return engine
}
