#!/bin/sh
# Builds the Linux .deb in an ubuntu:22.04 container: WebKitGTK does not
# cross-compile from a Mac. The image carries CI's packages, so the build has
# CI's glibc floor. node_modules is a container volume, so the Linux build
# never swaps the Host's macOS bindings for Linux ones.
set -e
IMAGE=demysto-linux-build

docker image inspect "$IMAGE" >/dev/null 2>&1 || docker build --platform linux/arm64 -t "$IMAGE" - <<'DOCKERFILE'
FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
      libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf libxdo-dev \
      build-essential curl ca-certificates file pkg-config \
 && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
 && apt-get install -y --no-install-recommends nodejs \
 && rm -rf /var/lib/apt/lists/*
RUN curl -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
ENV PATH=/root/.cargo/bin:$PATH
DOCKERFILE

docker run --rm --platform linux/arm64 \
  -v "$PWD":/src \
  -v demysto-linux-node-modules:/src/node_modules \
  -v demysto-linux-cargo-registry:/root/.cargo/registry \
  -v demysto-linux-npm:/root/.npm \
  -w /src -e CARGO_TARGET_DIR=/src/target/linux-arm64 \
  "$IMAGE" \
  bash -lc 'npm ci --no-audit --no-fund && npx tauri build --bundles deb --config "{\"bundle\":{\"createUpdaterArtifacts\":false}}"'
