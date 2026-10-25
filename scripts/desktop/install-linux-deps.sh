#!/usr/bin/env bash
# Dependencias de sistema para build/smoke/e2e AppImage Tauri v2 (ubuntu-22.04).
set -euo pipefail

sudo apt-get update
sudo apt-get install -y \
  build-essential \
  curl \
  wget \
  file \
  libwebkit2gtk-4.1-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf \
  libssl-dev \
  libgtk-3-dev \
  libxdo-dev \
  libfuse2 \
  desktop-file-utils \
  squashfs-tools \
  xvfb \
  xauth \
  webkit2gtk-driver \
  ripgrep \
  jq \
  rsync
