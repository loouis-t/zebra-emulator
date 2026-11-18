import { init, zplToBase64MultipleAsync } from 'zpl-renderer-js/external'
import wasmUrl from 'zpl-renderer-js/wasm?url'

const MM_PER_INCH = 25.4
const DOTS_PER_MM = 8

const ready = init({ wasmUrl })

export async function renderZpl(base64Zpl: string, size: { width: number; height: number }) {
  await ready
  const zpl = new TextDecoder().decode(Uint8Array.from(atob(base64Zpl), c => c.charCodeAt(0)))
  const images = await zplToBase64MultipleAsync(zpl, size.width * MM_PER_INCH, size.height * MM_PER_INCH, DOTS_PER_MM)
  return images.map(png => `data:image/png;base64,${png}`)
}
