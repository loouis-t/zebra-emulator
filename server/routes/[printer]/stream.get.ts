export default defineEventHandler((event) => {
  const stream = createEventStream(event)
  const unsubscribe = subscribe(getRouterParam(event, 'printer')!, zpl => stream.push(zpl))
  const ping = () => stream.push({ event: 'ping', data: '' })
  const keepAlive = setInterval(ping, 30_000)
  stream.onClosed(async () => {
    clearInterval(keepAlive)
    unsubscribe()
    await stream.close()
  })
  ping()
  return stream.send()
})
