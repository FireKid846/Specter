// ─── Voice Coach ──────────────────────────────────────────────────────────────
// Handles TTS — online uses CF Workers AI (MeloTTS), offline uses Web Speech API.

export interface VoiceConfig {
  enabled:  boolean
  online:   boolean    // Use CF Workers AI TTS
  rate:     number     // Speech rate (0.5–2.0)
  pitch:    number     // Pitch (0–2)
  volume:   number     // Volume (0–1)
}

export const DEFAULT_VOICE_CONFIG: VoiceConfig = {
  enabled: true,
  online:  true,
  rate:    1.0,
  pitch:   1.0,
  volume:  1.0,
}

/** Speak a message using Web Speech API (offline, in-browser). */
export function speakOffline(text: string, config: VoiceConfig = DEFAULT_VOICE_CONFIG): void {
  if (typeof window === 'undefined' || !window.speechSynthesis) return
  const utterance      = new SpeechSynthesisUtterance(text)
  utterance.rate       = config.rate
  utterance.pitch      = config.pitch
  utterance.volume     = config.volume
  window.speechSynthesis.cancel()
  window.speechSynthesis.speak(utterance)
}

/** Request TTS audio from the Specter API (online, high quality). */
export async function speakOnline(
  text:    string,
  apiUrl:  string,
  config:  VoiceConfig = DEFAULT_VOICE_CONFIG
): Promise<void> {
  try {
    const res = await fetch(`${apiUrl}/coach/voice`, {
      method:  'POST',
      headers: { 'Content-Type': 'application/json' },
      body:    JSON.stringify({ text }),
    })
    if (!res.ok) throw new Error('TTS API failed')
    const blob = await res.blob()
    const url  = URL.createObjectURL(blob)
    const audio = new Audio(url)
    audio.volume = config.volume
    await audio.play()
  } catch {
    // Fallback to offline TTS
    speakOffline(text, config)
  }
}
