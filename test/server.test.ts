import { afterAll, beforeAll, expect, test } from 'bun:test'
import type { Subprocess } from 'bun'

const port = 3100 + Math.floor(Math.random() * 800)
const url = `http://127.0.0.1:${port}`
let server: Subprocess

beforeAll(async () => {
  server = Bun.spawn([process.execPath, '.output/server/index.mjs'], {
    env: { ...process.env, PORT: String(port) },
    stdout: 'inherit',
    stderr: 'inherit',
  })
  for (let i = 0; i < 100; i++) {
    if (await fetch(url).then(() => true, () => false)) return
    await Bun.sleep(100)
  }
  throw new Error('server did not start')
}, 15_000)

afterAll(() => server.kill())

async function subscribe(printer: string) {
  const controller = new AbortController()
  const res = await fetch(`${url}/${printer}/stream`, { signal: controller.signal })
  const reader = res.body!.getReader()
  const next = async (timeoutMs = 5000) => {
    const timeout = Bun.sleep(timeoutMs).then(() => null)
    while (true) {
      const chunk = await Promise.race([reader.read(), timeout])
      if (!chunk || chunk.done) return null
      const data = new TextDecoder().decode(chunk.value).match(/^data: ?(\S+)$/m)?.[1]
      if (data) return Buffer.from(data, 'base64')
    }
  }
  return { res, next, close: () => controller.abort() }
}

const print = (printer: string, body: BodyInit) => fetch(`${url}/${printer}/pstprnt`, { method: 'POST', body })

test('print relays the raw bytes to the stream', async () => {
  const stream = await subscribe('p')
  expect(stream.res.headers.get('content-type')).toStartWith('text/event-stream')

  const zpl = Buffer.from('^XA\r\n^CI27^FO50,50^FD\xe9t\xe9^FS\n^XZ', 'latin1')
  expect((await print('p', zpl)).status).toBe(200)
  expect(await stream.next()).toEqual(zpl)
  stream.close()
})

test('prints only reach their own printer', async () => {
  const a = await subscribe('a')
  const b = await subscribe('b')
  await print('b', '^XA^XZ')
  expect((await b.next())?.toString()).toBe('^XA^XZ')
  expect(await a.next(300)).toBeNull()
  a.close()
  b.close()
})

test('cors allows cross-origin prints', async () => {
  const res = await fetch(`${url}/p/pstprnt`, {
    method: 'OPTIONS',
    headers: { origin: 'http://example.com', 'access-control-request-method': 'POST' },
  })
  expect(res.headers.get('access-control-allow-origin')).toBe('*')
})

test('pstprnt rejects other methods', async () => {
  expect((await fetch(`${url}/p/pstprnt`)).status).toBe(405)
})

test('serves the app on any printer path', async () => {
  for (const path of ['/', '/some-printer-1234']) {
    const res = await fetch(url + path)
    expect(res.status).toBe(200)
    expect(await res.text()).toContain('<div id="__nuxt">')
  }
})

test('serves the renderer wasm compressed', async () => {
  const assets = await Array.fromAsync(new Bun.Glob('*.wasm').scan('.output/public/_nuxt'))
  expect(assets.length).toBe(1)
  const res = await fetch(`${url}/_nuxt/${assets[0]}`, { headers: { 'accept-encoding': 'br, gzip' }, decompress: false })
  expect(res.status).toBe(200)
  expect(res.headers.get('content-encoding')).toBe('br')
  expect(res.headers.get('content-type')).toBe('application/wasm')
})
