type Subscriber = (zpl: Buffer) => void

const printers = new Map<string, Set<Subscriber>>()

export function publish(printer: string, zpl: Buffer) {
  printers.get(printer)?.forEach(send => send(zpl))
}

export function subscribe(printer: string, send: Subscriber) {
  const subscribers = printers.get(printer) ?? new Set()
  printers.set(printer, subscribers.add(send))
  return () => {
    subscribers.delete(send)
    if (!subscribers.size) printers.delete(printer)
  }
}
