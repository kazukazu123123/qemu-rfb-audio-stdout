# qemu-rfb-audio-stdout

This Rust program is a simple client that connects to a VNC server and captures audio from a QEMU-based server using its audio encoding.

## Features

- Connects to a VNC server and negotiates the initial protocol.
- Captures audio from the server using QEMU's audio encoding.
- Logs connection and audio-related events.

## Audio Data Format

- Audio data is captured in **16-bit signed PCM (`S16`)** format.
- The audio is captured in **stereo (2 channels)** at a sample rate of **48,000 Hz**.

## Notes

- Audio data will be printed to `stdout` when available.
- The program supports capturing audio in `S16` format, with stereo channels and a sample rate of 48,000 Hz.
- The program handles basic VNC protocol features, but may not support all VNC server configurations.

## Requirements

- Rust programming language (install from [here](https://www.rust-lang.org/tools/install))
- `clap` crate for argument parsing

## Installation

1. Clone or download the repository.
2. Build the project using Cargo:

   ```sh
   cargo build --release
   ```

3. Run the compiled binary.

## Usage

```sh
qemu-rfb-audio-stdout --address <address> --port <port>
```

### Arguments

- `--address` (`-a`): The VNC server's IP address (default: `127.0.0.1`).
- `--port` (`-p`): The VNC server's port (default: `5900`).

### Example

```sh
qemu-rfb-audio-stdout --address 192.168.1.100 --port 5900
```

## Output

### `stderr`

Logs events and errors:

- `EVT.CONNECTING`: Attempting to connect to the server.
- `EVT.CONNECTED`: Successfully connected to the server.
- `EVT.CONN_CLOSED`: Connection closed
- `EVT.LOG <message>`: General information (e.g., version, security type).
- `EVT.ERROR_LOG <message>`: Error messages (e.g., connection issues, authentication failures).
- `EVT.AUDIOSTART`: The audio stream has started.
- `EVT.AUDIOSTOP`: The audio stream has stopped.

### `stdout`

Audio data will be output to stdout when available.
