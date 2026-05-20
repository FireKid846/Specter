# Specter Engine Architecture

## Overview

Specter is a Rust chess engine compiled to WebAssembly for browser deployment,
with a native CLI binary for UCI-compatible GUIs.

## Package Structure

```
packages/engine/src/
├── board/          Board representation and position management
├── movegen/        Move generation (magic bitboards)
├── search/         PVS search with all pruning techniques
├── eval/           Hand-crafted evaluation (NNUE in Phase 2)
├── history/        Move ordering history tables
├── tt/             Transposition table
├── movepick/       Move picker and staging
├── personality/    Style and skill system
├── variants/       Variant-specific rules
├── uci/            UCI protocol handler
├── openingbook/    Built-in + Polyglot book
├── syzygy/         Tablebase probing (Fathom FFI)
└── wasm/           WebAssembly bindings
```

## Board Representation

Specter uses **bitboards** — each piece type and color is represented
as a 64-bit integer where each bit corresponds to a square.

```
Bit layout: a1=0, b1=1, ..., h1=7, a2=8, ..., h8=63
```

12 bitboards total: 6 piece types × 2 colors.

## Move Generation

**Magic bitboards** for sliding pieces (bishop, rook, queen):

```
index = ((occupancy & mask[sq]) * magic[sq]) >> (64 - bits[sq])
attacks = attack_table[sq][index]
```

This gives O(1) attack lookup — the fastest possible approach.

## Search Algorithm

**Principal Variation Search (PVS)** with iterative deepening:

1. Start at depth 1, increase to depth N
2. Use aspiration windows to narrow search
3. After first move: search remaining moves with null window `[α, α+1]`
4. Re-search with full window only if null window fails

**Pruning techniques (in order of application):**
- Transposition table cutoffs
- Reverse Futility Pruning (RFP)
- Null Move Pruning (NMP)
- Razoring
- Futility pruning
- Late Move Pruning (LMP)
- Late Move Reductions (LMR)
- Singular Extensions

## Evaluation

Phase 1 (current): Hand-crafted evaluation
- Material + piece-square tables (tapered MG/EG)
- Pawn structure (passed, isolated, doubled, backward)
- King safety (attack zone, pawn shelter, open files)
- Mobility (per-piece mobility tables)
- Threats (hanging pieces, pawn threats, attack pressure)

Phase 2: NNUE — `(768 → 384)x2 → 1` architecture

## Game Phase (Tapered Eval)

Score = (MG_score × phase + EG_score × (24 - phase)) / 24

Where phase is the sum of material phase weights:
- Knight/Bishop: 1 each
- Rook: 2
- Queen: 4
- Max phase: 24 (full material)
