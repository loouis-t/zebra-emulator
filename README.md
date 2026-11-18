# Zebra Emulator

This is a simple emulator for Zebra label printers.
Each virtual printer receives ZPL over HTTP, renders it to PNG and streams the result
live to the web app.

## Usage

```bash
./zebra-emulator --web-dir web/.output/public
```

Then open http://localhost:8080, pick a printer name and point your software at the
printer address shown on the page. Run `./zebra-emulator --help` for all options
(default label size, port, interface, web directory).

With Docker:

```bash
docker build -t zebra-emulator . && docker run -p 8080:8080 zebra-emulator
```

## API

| Route                    | Description                                  |
|--------------------------|----------------------------------------------|
| `POST /{printer}/pstprnt` | Render the ZPL body and push it to viewers  |
| `GET /{printer}/stream`   | Server-sent events, one base64 PNG per label |
| `GET/POST /{printer}/config` | Label size in inches: `{"width":2.25,"height":1.25}` |

## Test

```bash
curl -d '^XA^FO50,50^ADN,36,20^FDHello World!^FS^XZ' http://localhost:8080/my-printer/pstprnt
```
