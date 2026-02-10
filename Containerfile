# --- Stage 1: Build ---
FROM docker.io/nvidia/cuda:12.9.1-cudnn-devel-ubuntu24.04 AS builder

ARG CUDA_ARCH=61
ENV DEBIAN_FRONTEND=noninteractive \
    PATH="/root/.cargo/bin:${PATH}" \
    CMAKE_CUDA_ARCHITECTURES=${CUDA_ARCH}

# 1. SETUP: System Deps + GTK4 Layer Shell + Rust
# Combined to create a single cached layer for the environment.
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    pkg-config \
    libssl-dev \
    libclang-dev \
    cmake \
    curl \
    git \
    libasound2-dev \
    libgtk-4-dev \
    meson \
    ninja-build \
    gobject-introspection \
    libgirepository1.0-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    # Build gtk4-layer-shell
    && git clone --depth 1 --branch v1.3.0 https://github.com/wmww/gtk4-layer-shell.git /tmp/gtk4-layer-shell \
    && cd /tmp/gtk4-layer-shell \
    && meson setup build --prefix=/usr -Dvapi=false \
    && ninja -C build \
    && ninja -C build install \
    && cd / \
    && rm -rf /tmp/gtk4-layer-shell \
    # Install Rust
    && curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y \
    && rustup component add rustfmt clippy

WORKDIR /app

# 2. DEPENDENCIES: Manifests
# We must keep these separate to preserve the dependency compilation cache.
# If we merged them, any change to a Cargo.toml would invalidate the whole dependency build.
COPY Cargo.toml Cargo.lock ./
COPY stt-daemon/Cargo.toml ./stt-daemon/
COPY stt-client/Cargo.toml ./stt-client/
COPY stt-model-manager/Cargo.toml ./stt-model-manager/

# 3. CACHE: Compile dependencies with dummy sources
RUN mkdir -p stt-daemon/src stt-client/src stt-model-manager/src && \
    echo "fn main() {}" > stt-daemon/src/main.rs && \
    echo "fn main() {}" > stt-client/src/main.rs && \
    echo "fn main() {}" > stt-model-manager/src/main.rs && \
    cargo build --release --workspace && \
    rm -rf stt-daemon/src stt-client/src stt-model-manager/src

# 4. SOURCE: Copy entire project context
# Using 'COPY . .' is much faster (1 layer) than copying folders individually.
# .dockerignore ensures we don't copy 'target/', '.git/', etc.
COPY . .

# 5. BUILD: Final compilation
# We force 'touch' to ensure Cargo detects file changes over the dummy files.
RUN touch stt-daemon/src/main.rs stt-client/src/main.rs stt-model-manager/src/main.rs && \
    cargo clippy --release --workspace -- -D warnings && \
    cargo build --release --workspace

# --- Stage 2: Runtime ---
FROM docker.io/nvidia/cuda:12.9.1-cudnn-runtime-ubuntu24.04

WORKDIR /app

# 6. RUNTIME ENV: Install libs + Configure
RUN apt-get update && apt-get install -y --no-install-recommends \
    libasound2t64 \
    libasound2-plugins \
    libgtk-4-1 \
    && rm -rf /var/lib/apt/lists/* \
    && echo 'pcm.!default { type pulse }' > /etc/asound.conf \
    && echo 'ctl.!default { type pulse }' >> /etc/asound.conf

# 7. ARTIFACTS: Gather all artifacts in one go
# We copy libs and binaries to a temporary staging area in a SINGLE layer
# to avoid multiple 'commit' overheads.
COPY --from=builder \
    /usr/lib/x86_64-linux-gnu/libgtk4-layer-shell.so* \
    /usr/lib/x86_64-linux-gnu/girepository-1.0/Gtk4LayerShell-1.0.typelib \
    /app/target/release/stt-daemon \
    /app/target/release/stt-client \
    /app/target/release/stt-model-manager \
    /tmp/artifacts/

# 8. INSTALL: Move artifacts to final locations
RUN mkdir -p /usr/lib/x86_64-linux-gnu/girepository-1.0/ && \
    mv /tmp/artifacts/libgtk4-layer-shell* /usr/lib/x86_64-linux-gnu/ && \
    mv /tmp/artifacts/Gtk4LayerShell-1.0.typelib /usr/lib/x86_64-linux-gnu/girepository-1.0/ && \
    mv /tmp/artifacts/stt-* . && \
    rm -rf /tmp/artifacts

ENTRYPOINT ["./stt-daemon"]