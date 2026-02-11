# Contributing to Telora

Thank you for your interest in improving Telora!

## Project Structure

- `telora-daemon`: Rust daemon handling audio input and Whisper transcription (CUDA).
- `telora`: GTK4 client for UI feedback and control.
- `telora-models`: Tool for managing Whisper models.
- `pkg/`: Arch Linux packaging files.
- `scripts/`: Build and verification scripts.

## Development Workflow

### 1. Prerequisites
- Rust (Edition 2024)
- Podman (for containerized builds)
- GTK4 and Layer Shell libraries (if building locally)
- CUDA Toolkit (for GPU acceleration)

### 2. Building
The recommended way to build is using the provided script, which ensures a consistent environment:
```bash
./scripts/build
```

### 3. Local Testing
You can run the binaries directly from the `bin/` directory after building:
```bash
# Start the daemon
./bin/telora-daemon --model ./models/ggml-base.bin

# In another terminal, run the client
./bin/telora
```

## Coding Standards

- **Rust**: Follow idiomatic Rust patterns. Use `cargo fmt` and `cargo clippy`.
- **Commits**: Use descriptive commit messages. Follow the format: `type: Description` (e.g., `fix: Audio buffer overflow`).
- **Privacy**: Never introduce code that logs transcriptions or sends data to external servers. Telora is strictly local.

## Debugging

To enable debug logs, use the `RUST_LOG` environment variable:
```bash
RUST_LOG=debug ./bin/telora-daemon
```

You can also override the model path for testing:
```bash
TELORA_MODEL_PATH=/path/to/model.bin ./bin/telora-daemon
```

## Questions?
Feel free to open an issue or a discussion on GitHub.
