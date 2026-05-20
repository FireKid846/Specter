export function validateFen(fen: string): boolean {
  const parts = fen.trim().split(/\s+/)
  if (parts.length < 4) return false
  const rows = parts[0].split('/')
  if (rows.length !== 8) return false
  return true
}

export async function validateRequest(
  request: Request,
  required: string[]
): Promise<{ valid: boolean; body: any; missing: string[] }> {
  const body = await request.json().catch(() => ({})) as Record<string, any>
  const missing = required.filter(k => !body[k])
  return { valid: missing.length === 0, body, missing }
}
