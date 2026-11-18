export default defineEventHandler(async (event) => {
  if (handleCors(event, { origin: '*', methods: ['POST'], allowHeaders: '*', maxAge: '3600' })) return
  assertMethod(event, 'POST')
  publish(getRouterParam(event, 'printer')!, (await readRawBody(event, false)) ?? Buffer.alloc(0))
  return ''
})
