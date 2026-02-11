#!/bin/bash
set -e

# Ensure environment is set up
if [ -f "./scripts/compatibility/setup-env.sh" ]; then
    ./scripts/compatibility/setup-env.sh
fi

echo "--> Verifying Telora binaries..."

for bin in telora-daemon telora telora-models; do
    if [ ! -f "bin/$bin" ]; then
        echo "Error: bin/$bin not found!"
        exit 1
    fi
done

echo "--> [1/3] Testing telora-models..."
./bin/telora-models --version

echo "--> [2/3] Testing telora-daemon..."
./bin/telora-daemon --version

echo "--> [3/3] Testing telora..."
./bin/telora --version

echo "--> Verification successful!"
