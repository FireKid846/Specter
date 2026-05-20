# Specter API Endpoints

Base URL: `https://specter-api.your-domain.workers.dev`

## POST /move

Get the best move for a position.

**Request:**
```json
{
  "fen":    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
  "timeMs": 100,
  "depth":  0,
  "elo":    1500,
  "style":  "balanced"
}
```

**Response:**
```json
{
  "bestMove": "e2e4",
  "eval":     23,
  "depth":    12,
  "pv":       ["e2e4", "e7e5", "g1f3"],
  "nodes":    145230,
  "timeMs":   98
}
```

## POST /analysis/game

Full game review.

**Request:**
```json
{ "pgn": "1. e4 e5 2. Nf3 Nc6 ..." }
```

## POST /analysis/position

Single position analysis.

**Request:**
```json
{ "fen": "...", "depth": 18 }
```

## GET /puzzles/daily

Returns today's daily puzzle.

## GET /puzzles/random?rating=1500&theme=fork

Random puzzle by rating/theme.

## POST /puzzles/validate

Validate a puzzle solution.

## POST /coach/text

Get AI coaching message for a move.

## POST /coach/voice

Get TTS audio for a coaching message.

## GET /openings/identify

Identify an opening by FEN.

## GET /openings/search?q=sicilian

Search openings by name.

## POST /rating/calculate

Calculate new ELO or Glicko-2 rating.
