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

You can configure the daemon using a TOML file. The daemon looks for configuration in the following order:

1.  **CLI Arguments**: (e.g., `--config my_config.toml` or `--language en`)
2.  **User Config**: `~/.config/stt-assistant/config.toml`
3.  **System Config**: `/etc/stt-assistant.toml`
4.  **Environment Variables**: (e.g., `STT_LANGUAGE=fr`)

### Example Configuration (`config.toml`)

```toml
# Path to the model file.
# Can be an absolute path, or relative to:
# - $HOME/.local/share/stt-assistant/models/
# - /usr/share/stt-assistant/models/
# - ./models/
model_path = "ggml-base.bin"

# Language code (e.g., "es", "en", "fr")
# This is passed to the Whisper model.
language = "es"

# Maximum recording time in seconds.
# The daemon will automatically stop and process the audio if this limit is reached.
# Default is 300 seconds (5 minutes). Set to a higher value for long dictations,
# or lower to prevent memory abuse.
max_recording_seconds = 300
```

## Customizing Systemd Services

If you need to change how the services start (e.g., adding environment variables like `RUST_LOG`), the best practice is to use a **drop-in override** rather than copying the entire file.

### Example: Enable Debug Logging

1.  Create an override for the user service:
    ```bash
    systemctl --user edit stt-daemon.service
    ```
2.  Add your changes in the editor that opens:
    ```ini
    [Service]
    Environment=RUST_LOG=debug
    ```
3.  Save and exit. Systemd will automatically reload.
4.  Restart the service:
    ```bash
    systemctl --user restart stt-daemon.service
    ```

This method preserves your changes even if the main package updates the service file.

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

## Daemon Status & Monitoring

You can check the real-time status of the audio daemon (PID, current model, language, state, etc.) by running:

```bash
stt-daemon status
```

**Example Output:**

```text
STT Daemon Status
ACTIVE     PID        MODEL                          LANG       MAX_SEC    STATE
---------- ---------- ------------------------------ ---------- ---------- ---------------
YES        1234       ggml-base.bin                  es         300        Idle

Full Model Path: /usr/share/stt-assistant/models/ggml-base.bin
```

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