#!/usr/bin/env bash
# verify.sh - Runs automated linkage and help tests on all components
set -euo pipefail

# Ensure we are in the project root
cd "$(dirname "$0")/../.."

echo "--> Starting verification tests..."

# Unified export for shared libraries
CUDA_PATH="/opt/cuda/targets/x86_64-linux/lib"
export LD_LIBRARY_PATH="$(pwd)/bin:${CUDA_PATH}:${LD_LIBRARY_PATH:-}"

# Check binaries exist
for bin in stt-daemon stt-client stt-model-manager; do
    if [ ! -f "bin/$bin" ]; then
        echo "!! Error: Binary bin/$bin not found. Run ./scripts/build first."
        exit 1
    fi
done

echo "--> [1/3] Testing stt-model-manager..."
./bin/stt-model-manager --version

echo "--> [2/3] Testing stt-daemon..."
./bin/stt-daemon --version

echo "--> [3/3] Testing stt-client..."
./bin/stt-client --version

echo "--> ALL TESTS PASSED!"
