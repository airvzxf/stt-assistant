# Contributing

This guide is for developers who want to modify and test STT Assistant on Arch Linux.

## Prerequisites

To build and run this project, you need:
- **Podman**: For the containerized build (recommended).
- **Nvidia Drivers**: Since the daemon uses CUDA.
- **GTK4**: For the client.

### Arch Linux Dependencies
Install the required runtime libraries:
```bash
sudo pacman -S gtk4 alsa-lib gcc-libs
# Note: gtk4-layer-shell may need to be installed from AUR (e.g., yay -S gtk4-layer-shell)
```

## Build & Installation (Developer Mode)

### 1. Build Binaries
The easiest way to build without worrying about local toolchains (CUDA/Clang) is using the provided script:
```bash
./scripts/build
```
This puts the binaries in `bin/`.

### 2. Local Installation (User level)
For development, you might prefer installing to your home directory instead of system-wide:
```bash
make PREFIX=$HOME/.local install
```
Then update your systemd user units:
```bash
systemctl --user daemon-reload
```

### 3. System Packaging (Professional)
To verify the Arch Linux package:
```bash
cd pkg
makepkg -si
```

## Structure

- `stt-daemon`: Rust daemon handling audio input and Whisper transcription (CUDA).
- `stt-client`: GTK4 client for UI feedback.
- `stt-model-manager`: Tool for managing Whisper models.
- `scripts/build`: Main build script (wraps Podman).
- `pkg/`: Arch Linux packaging files.
- `systemd/`: Service units.

## Working with Models (Development)

During development, you can test specific models without installing them:

1.  **Place models in the root `models/` directory**: The daemon will find them there if they are not in your home directory.
2.  **Use explicit paths**:
    ```bash
    ./bin/stt-daemon --model ./my-experimental-models/ggml-base.bin
    ```
3.  **Override via Environment**:
    ```bash
    STT_MODEL_PATH=/path/to/model.bin ./bin/stt-daemon
    ```

## Coding Standards
- Run `cargo fmt` before committing.
- Ensure `cargo clippy` passes (it is checked during `./scripts/build`).
