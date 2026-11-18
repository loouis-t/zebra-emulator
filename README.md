# Zebra Emulator

This is a simple emulator for Zebra label printers.
The server only relays the ZPL each virtual printer receives; labels are rendered in the
browser by [zpl-renderer-js](https://github.com/Fabrizz/zpl-renderer-js) (WebAssembly).

## Usage

```bash
bun install
bun run build
bun run start
```

Then open http://localhost:3000, pick a printer name and point your software at the
printer address shown on the page. The label size is set per printer in the page.
Use `bun run dev` while developing and `bun run test` to build and run the tests.

To deploy on Coolify, use the Railpack build pack with port 3000.

## API

| Route                     | Description                                          |
|---------------------------|------------------------------------------------------|
| `POST /{printer}/pstprnt` | Relay the ZPL body to that printer's viewers         |
| `GET /{printer}/stream`   | Server-sent events, one base64 ZPL payload per print |

## Example

```bash
curl -d '^XA^FO50,50^ADN,36,20^FDHello World!^FS^XZ' http://localhost:3000/my-printer/pstprnt
```
