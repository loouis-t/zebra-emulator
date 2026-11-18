export default defineEventHandler((event) => {
  const stream = createEventStream(event)
  const unsubscribe = subscribe(getRouterParam(event, 'printer')!, zpl => stream.push(zpl.toString('base64')))
  const ping = () => stream.push({ event: 'ping', data: '' })
  const keepAlive = setInterval(ping, 30_000)
  stream.onClosed(() => {
    clearInterval(keepAlive)
    unsubscribe()
  })
  ping()
  return stream.send()
})
