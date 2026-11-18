export default defineEventHandler(async (event) => {
  if (handleCors(event, { origin: '*', methods: ['POST'], allowHeaders: '*', maxAge: '3600' })) return
  assertMethod(event, 'POST')
  const zpl = await readRawBody(event, false)
  publish(getRouterParam(event, 'printer')!, Buffer.from(zpl ?? []).toString('base64'))
  return ''
})
