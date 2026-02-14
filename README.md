# Zebra Emulator

This is a simple emulator for Zebra label printers.
It listens for incoming connections on a specified port and renders the received ZPL commands as images.

The temporary image files are stored in `/tmp`, and are opened automatically using the default image
viewer on your system.

## Usage

To run the emulator to render 2,25x1,25 inch labels, use the following command:

```bash
./zebra-emulator
```

### CLI Options

| Option              | Description           | Default   |
|---------------------|-----------------------|-----------|
| `-x`, `--width`     | Label width (inches)  | 2.25      |
| `-y`, `--height`    | Label height (inches) | 1.25      |
| `-p`, `--port`      | Port to listen on     | 8080      |
| `-i`, `--interface` | Interface to bind to  | 127.0.0.1 |

## Test

You can test the emulator by sending a simple HTTP POST request with ZPL commands.

```bash
curl -v -H 'Content-Length: 42' \
     -H 'Content-Type: text/plain;charset=UTF-8' \
     -d '^XA^FO50,50^ADN,36,20^FDHello World!^FS^XZ' \
     http://localhost:8080/pstprnt
```
