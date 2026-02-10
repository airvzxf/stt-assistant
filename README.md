# STT Assistant

A professional Speech-to-Text Assistant for Linux, featuring a high-performance Rust daemon using Whisper (CUDA-accelerated) and a lightweight GTK4 client.

## Features

- **Daemon**: Rust-based, using `whisper-rs` for local, privacy-focused transcription. Now configurable via CLI or TOML.
- **Model Manager**: Integrated CLI tool to download and manage Whisper models (Tiny, Base, Small, etc.).
- **Client**: GTK4 Layer Shell interface for seamless desktop integration.
- **Packaging**: Ready for Arch Linux (PKGBUILD provided).

## Installation (Arch Linux)

This project uses a containerized build process to ensure CUDA and GTK compatibility.

### 1. Build the binaries
You must build the binaries first using Podman:
```bash
./scripts/build
```

### 2. Install the package
You can then install the package using the provided PKGBUILD:
```bash
cd pkg
makepkg -si
```
*Dependencies from official Arch repos (`gtk4`, `gtk4-layer-shell`, `cuda`, etc.) will be installed automatically.*

## Configuration

You can configure the assistant using `stt-assistant.toml`. The daemon searches for this file in the current directory by default.

```toml
model_path = "ggml-base.bin"
language = "es"
# Safety limit for recording duration (in seconds)
max_recording_seconds = 300
```

You can also override these settings via CLI arguments when running `stt-daemon`:
```bash
stt-daemon --max-recording-seconds 600 --language en
```

## Client CLI & Controls

The `stt-client` now supports CLI commands for integration with shortcuts or scripts:

```bash
# Toggle recording and TYPE the result
stt-client toggle-type

# Toggle recording and COPY the result to clipboard
stt-client toggle-copy

# Cancel current recording
stt-client cancel
```

Run `stt-client --help` for more details.

## Security & Privacy

- **Memory Protection**: The daemon enforces a memory limit on audio buffers (configurable via `max_recording_seconds`) to prevent OOM crashes.
- **Socket Security**: IPC sockets are restricted to the owner (`0600`), preventing unauthorized local access.
- **Privacy**: Transcriptions are processed locally and never logged to disk or system logs. Temporary file communication has been replaced with secure direct memory transfer.

## Model Management

Use `stt-model-manager` to download and manage Whisper models:

```bash
# List available and installed models
stt-model-manager list

# Download a predefined model
stt-model-manager download base

# Download ANY model from whisper.cpp HuggingFace repo (e.g. large-v3-turbo-q8_0)
stt-model-manager download large-v3-turbo-q8_0

# Download from a custom URL
stt-model-manager download --url https://example.com/models/custom-whisper.bin

# Specify a custom output name
stt-model-manager download base --out my-model.bin

# Download a model for all users (requires sudo)
sudo stt-model-manager download base --global
```

### Model Resolution (Precedence)

When you specify a model (via CLI `--model` or TOML `model_path`), the daemon resolves the path using the following priority:

1.  **Explicit Path**: If you provide a full or relative path (e.g., `./my-models/tiny.bin`), it is used directly.
2.  **User Models**: `~/.local/share/stt-assistant/models/`
3.  **System Models**: `/usr/share/stt-assistant/models/`
4.  **Local Development**: `./models/` (current working directory)

**Note:** If two models have the same name, the **User** version shadows the **System** version.

## Usage

Start the assistant (this will automatically start the background daemon):

```bash
systemctl --user enable --now stt-assistant.service
```

The `stt-assistant` service manages the UI and requires `stt-daemon` (the audio engine), which systemd will handle for you.

## Development

For detailed development instructions, local installation to `~/.local`, and coding standards, see [CONTRIBUTING.md](CONTRIBUTING.md).

## License

[GNU AFFERO | Version 3](LICENSE)