#!/usr/bin/env bash
# Shared Ubuntu native dependencies. SDL backends stay enabled.
set -euo pipefail
packages=(
  clang
  cmake
  fonts-dejavu-core
  libasound2-dev
  libc++-dev
  libc++abi-dev
  libdecor-0-dev
  libdrm-dev
  libegl-dev
  libfontconfig1-dev
  libffi-dev
  libfreetype-dev
  libgbm-dev
  libgl-dev
  libgl1-mesa-dri
  libpipewire-0.3-dev
  libpulse-dev
  libsdl2-dev
  libudev-dev
  libusb-1.0-0-dev
  libwayland-dev
  libx11-dev
  libxcb-cursor0
  libxcb1-dev
  libxcursor-dev
  libxfixes-dev
  libxi-dev
  libxkbcommon-dev
  libxrandr-dev
  libxss-dev
  libxtst-dev
  mesa-vulkan-drivers
  pkg-config
  x11-apps
  xauth
  xclip
  xvfb
)
if [[ "${1:-}" == --list ]]; then
  printf '%s\n' "${packages[@]}"
  exit 0
fi
if [[ $# != 0 ]]; then
  echo "usage: $0 [--list]" >&2
  exit 2
fi
if [[ "$(uname -s)" != Linux ]]; then
  echo "Native Linux dependency installation requires Linux" >&2
  exit 2
fi
sudo apt-get update
sudo apt-get install -y --no-install-recommends "${packages[@]}"
