# --- Stage 1: Build ---
FROM docker.io/nvidia/cuda:12.9.1-cudnn-devel-ubuntu24.04 AS builder

ARG CUDA_ARCH=61
ENV DEBIAN_FRONTEND=noninteractive

# Install dependencies
RUN apt-get update && apt-get install -y \
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
    && rm -rf /var/lib/apt/lists/*

# Build gtk4-layer-shell from source
RUN git clone --depth 1 --branch v1.3.0 https://github.com/wmww/gtk4-layer-shell.git /tmp/gtk4-layer-shell && \
    cd /tmp/gtk4-layer-shell && \
    meson setup build --prefix=/usr -Dvapi=false && \
    ninja -C build && \
    ninja -C build install && \
    rm -rf /tmp/gtk4-layer-shell

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"
RUN rustup component add rustfmt clippy

WORKDIR /app

# Cache dependencies by copying manifests first
COPY Cargo.toml Cargo.lock ./
COPY stt-daemon/Cargo.toml ./stt-daemon/
COPY stt-client/Cargo.toml ./stt-client/

# Create dummy sources for dependency pre-compilation
RUN mkdir -p stt-daemon/src stt-client/src && \
    echo "fn main() {}" > stt-daemon/src/main.rs && \
    echo "fn main() {}" > stt-client/src/main.rs

ENV CMAKE_CUDA_ARCHITECTURES=${CUDA_ARCH}
RUN cargo build --release --workspace
RUN rm -rf stt-daemon/src stt-client/src

# Copy real source code
COPY stt-daemon/src ./stt-daemon/src
COPY stt-client/src ./stt-client/src

# Touch to force rebuild
RUN touch stt-daemon/src/main.rs stt-client/src/main.rs

# Validate with Clippy
RUN cargo clippy --release --workspace -- -D warnings

# Final build
RUN cargo build --release --workspace

# --- Stage 2: Runtime ---
FROM docker.io/nvidia/cuda:12.9.1-cudnn-runtime-ubuntu24.04

RUN apt-get update && apt-get install -y \
    libasound2t64 \
    libasound2-plugins \
    libgtk-4-1 \
    && rm -rf /var/lib/apt/lists/*

# Copy gtk4-layer-shell library from builder
COPY --from=builder /usr/lib/x86_64-linux-gnu/libgtk4-layer-shell* /usr/lib/x86_64-linux-gnu/
COPY --from=builder /usr/lib/x86_64-linux-gnu/girepository-1.0/Gtk4LayerShell-1.0.typelib /usr/lib/x86_64-linux-gnu/girepository-1.0/

WORKDIR /app

# Copy binaries from builder
COPY --from=builder /app/target/release/stt-daemon .
COPY --from=builder /app/target/release/stt-client .

# Configure ALSA to use PulseAudio
RUN echo 'pcm.!default { type pulse }' > /etc/asound.conf && \
    echo 'ctl.!default { type pulse }' >> /etc/asound.conf

ENTRYPOINT ["./stt-daemon"]