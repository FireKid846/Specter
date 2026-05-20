# Specter Engine

A high-performance, fully configurable chess engine built in Rust.

## Features

- Bitboard representation with magic bitboard attack generation
- Principal Variation Search (PVS) with iterative deepening
- Full Stockfish-style move ordering (continuation history, correction history, killers)
- Lazy SMP multi-threading (CLI)
- Syzygy tablebase support
- WASM build for browser deployment
- UCI protocol + custom JS API
- Configurable playing personality (style, ELO, blunder rate)
- Variant support: Standard, Chess960, Four-Player, Three-Check, Horde, Atomic

## Packages

| Package | Description |
|---|---|
| `packages/engine` | Rust core engine → WASM |
| `packages/cli` | UCI binary (MIT licensed) |
| `packages/npm` | TypeScript WASM wrapper |
| `packages/api` | Cloudflare Workers REST API |
| `packages/chess-data` | PGN, ECO, rating |
| `packages/analysis` | Review, puzzles, coach |

## Build

```bash
# Build engine
cargo build --release -p specter-engine

# Build WASM
wasm-pack build packages/engine --target web

# Build CLI
cargo build --release -p specter-cli

# Run tests
cargo test

# Run perft
cargo test perft
```

## License

Engine source: Proprietary  
CLI source: MIT
