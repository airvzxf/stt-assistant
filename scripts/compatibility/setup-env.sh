#!/usr/bin/env bash
# setup-env.sh - Smart dependency installer for validated distributions
set -euo pipefail

echo "--> Detecting distribution..."

if [ -f /etc/debian_version ]; then
    echo "--> Distribution: Debian/Ubuntu family"
    sudo apt-get update
    sudo apt-get install -y \
        libgtk-4-1 \
        libadwaita-1-0 \
        libasound2t64 \
        libasound2-plugins \
        wl-clipboard \
        wtype \
        libgl1

elif [ -f /etc/fedora-release ]; then
    echo "--> Distribution: Fedora family"
    sudo dnf install -y \
        gtk4 \
        libadwaita \
        alsa-lib \
        pipewire-alsa \
        mesa-libGL \
        wl-clipboard \
        wtype

elif [ -f /etc/arch-release ]; then
    echo "--> Distribution: Arch Linux"
    sudo pacman -Syu --noconfirm \
        gtk4 \
        libadwaita \
        alsa-lib \
        pipewire-alsa \
        mesa-libGL \
        wl-clipboard \
        wtype

else
    echo "!! Error: Distribution not supported by setup-env.sh"
    echo "Please install dependencies manually: GTK4, libadwaita, ALSA, wl-clipboard, wtype."
    exit 1
fi

echo "--> Dependencies installed successfully."
