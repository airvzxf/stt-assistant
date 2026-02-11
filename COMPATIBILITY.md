# Linux Distribution Compatibility Matrix

This project uses an **Infrastructure as Code (IaC)** approach to ensure portability across the major Linux distribution families. We use `distrobox` to create isolated environments that share the host's GPU (NVIDIA/CUDA), Audio (Pipewire/ALSA), and Display Server (Wayland/X11).

## Verified Environments

The following distributions are officially supported and tested using our automated test suite:

| Distribution | Status | Family | Features Tested |
|--------------|--------|--------|-----------------|
| **Arch Linux** | ✅ Pass | Independent | GPU, Audio, GUI, Typing |
| **Fedora (Latest)** | ✅ Pass | RHEL/Fedora | GPU, Audio, GUI, Typing |
| **Debian (Stable)** | ✅ Pass | Debian/Ubuntu | GPU, Audio, GUI, Typing |

## How to Run Tests

To verify compatibility on your local machine, follow these steps:

### 1. Build the Binaries
First, ensure you have the latest standalone binaries:
```bash
./scripts/build
```

### 2. Assemble the Test Matrix
Create the verified containers using Distrobox (requires `podman` or `docker`):
```bash
distrobox-assemble create --replace --file scripts/compatibility/distrobox.ini
```

### 3. Run Automated Verification
You can run the verification script inside any of the containers. The script will automatically install dependencies using `setup-env.sh` on the first run:
```bash
# Example for Debian
distrobox enter telora-debian -- ./scripts/compatibility/verify.sh

# Example for Fedora
distrobox enter telora-fedora -- ./scripts/compatibility/verify.sh

# Example for Arch Linux
distrobox enter telora-arch -- ./scripts/compatibility/verify.sh
```

### 4. Full Stack Simulation
To test the full interaction (Daemon + UI) and simulate a transcription session within a container:

```bash
# This will run the daemon, start a recording, wait 4s, and stop it.
distrobox enter telora-fedora -- ./scripts/compatibility/simulate.sh
```

> [!TIP]
> If the simulation fails or hangs in an infinite loop, you can restore the container state by running:
> `podman container restart telora-fedora` (or the corresponding container name).

## Known Limitations

- **Alpine Linux / musl:** Pre-compiled binaries are built against `glibc` (Ubuntu base). They will not run on `musl`-based systems like Alpine without a compatibility layer or static recompilation.
- **Minimal Distros:** Some very minimal distributions (like Slackware base images) may lack the necessary user-management tools required for full Distrobox integration.

## Adding a New Distribution
To add a new distribution to the matrix:
1. Add a new section to `scripts/compatibility/distrobox.ini`.
2. Update `scripts/compatibility/setup-env.sh` with the corresponding package manager commands.
3. Run the verification script to confirm functionality.
