# API Reference

## `createEngine(): Promise<SpectorEngine>`
Creates and initializes a new engine instance.

## `SpectorEngine`

### `init(): Promise<void>`
Initialize the WASM engine. Called automatically by `createEngine()`.

### `setPosition(fen: string): void`
Set the board position from a FEN string.

### `makeMove(uci: string): boolean`
Apply a UCI move (e.g. `"e2e4"`). Returns `false` if the move is illegal.

### `getBestMove(config?: SpectorConfig): Promise<MoveResult>`
Search for the best move.

### `getLegalMoves(): string[]`
Returns all legal moves in UCI notation.

### `getGameState(): GameState`
Returns the current game state.

### `getFen(): string`
Returns the current position as a FEN string.

### `evaluate(): number`
Returns the static evaluation in centipawns (positive = good for side to move).

### `setElo(elo: number): void`
Set the engine's skill level by ELO rating.

### `reset(): void`
Reset to the starting position.
