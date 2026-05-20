// Loads the WASM binary — works in browser and Node.js

let wasmModule: any = null

export async function loadWasm(): Promise<any> {
  if (wasmModule) return wasmModule

  // Browser: import from bundled wasm
  if (typeof window !== 'undefined') {
    const { default: init, SpectorEngine } = await import('../wasm/specter.js' as any)
    await init()
    wasmModule = SpectorEngine
    return wasmModule
  }

  // Node.js
  const { SpectorEngine } = await import('../wasm/specter.js' as any)
  wasmModule = SpectorEngine
  return wasmModule
}
