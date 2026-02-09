# STT Assistant

A professional Speech-to-Text Assistant for Linux, featuring a high-performance Rust daemon using Whisper (CUDA-accelerated) and a lightweight GTK4 client.

## Features

- **Daemon**: Rust-based, using `whisper-rs` for local, privacy-focused transcription.
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