import type { EcoEntry } from '../types'
import ECO_DATA from './data/eco.json'

// ─── ECO Lookup ───────────────────────────────────────────────────────────────

const ecoByFen   = new Map<string, EcoEntry>()
const ecoByCode  = new Map<string, EcoEntry>()
let initialized  = false

function init() {
  if (initialized) return
  for (const entry of ECO_DATA as EcoEntry[]) {
    ecoByFen.set(normalizeFen(entry.fen), entry)
    ecoByCode.set(entry.eco, entry)
  }
  initialized = true
}

/** Normalize FEN to just piece placement + side + castling + ep (drop clocks). */
function normalizeFen(fen: string): string {
  return fen.split(' ').slice(0, 4).join(' ')
}

/** Look up opening by position FEN. Returns null if not found. */
export function lookupByFen(fen: string): EcoEntry | null {
  init()
  return ecoByFen.get(normalizeFen(fen)) ?? null
}

/** Look up opening by ECO code (e.g. "B90"). */
export function lookupByCode(eco: string): EcoEntry | null {
  init()
  return ecoByCode.get(eco.toUpperCase()) ?? null
}

/** Search openings by name fragment. Returns up to `limit` results. */
export function searchByName(query: string, limit = 10): EcoEntry[] {
  init()
  const q = query.toLowerCase()
  const results: EcoEntry[] = []
  for (const entry of ECO_DATA as EcoEntry[]) {
    if (entry.name.toLowerCase().includes(q)) {
      results.push(entry)
      if (results.length >= limit) break
    }
  }
  return results
}

/** Get all openings in a given ECO category (e.g. "B" = all B00-B99). */
export function byCategory(letter: string): EcoEntry[] {
  init()
  const l = letter.toUpperCase()
  return (ECO_DATA as EcoEntry[]).filter(e => e.eco.startsWith(l))
}

/** Returns the opening name for a FEN, or null. */
export function getOpeningName(fen: string): string | null {
  return lookupByFen(fen)?.name ?? null
}
