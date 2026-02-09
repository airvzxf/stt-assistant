#!/usr/bin/env bash
set -euo pipefail

# Build script for STT Rust workspace
IMAGE_NAME="stt-daemon:latest"

# Detect GPU architecture (GTX 1080 -> sm_61)
DETECTED_ARCH="61"
if command -v nvidia-smi &> /dev/null; then
    RAW_ARCH=$(nvidia-smi --query-gpu=compute_cap --format=csv,noheader | head -n 1 | tr -d '.')
    if [[ -n "${RAW_ARCH}" ]]; then
        DETECTED_ARCH="${RAW_ARCH}"
        echo "--> GPU architecture detected: ${DETECTED_ARCH}"
    fi
fi

TARGET_ARCH="${1:-${DETECTED_ARCH}}"
echo "--> Building container for architecture: ${TARGET_ARCH}"

# Format source files
echo "--> Formatting source files..."
time podman run --rm \
    -v "$(pwd):/app:z" \
    -w /app \
    docker.io/library/rust:latest \
    bash -c "rustup component add rustfmt && cargo fmt --all"

# Build container image
time podman build \
    --build-arg CUDA_ARCH="${TARGET_ARCH}" \
    -t "${IMAGE_NAME}" .

# Extract binaries
echo "--> Extracting binaries..."
mkdir -p bin
time podman run --rm \
    --entrypoint sh \
    -v "$(pwd):/app_out" \
    "${IMAGE_NAME}" \
    -c "cp /app/stt-daemon /app_out/bin/stt-daemon && cp /app/stt-client /app_out/bin/stt-client"

echo "--> Done! Binaries extracted to bin/"
echo "--> Run with: ./stt_ctl start"
