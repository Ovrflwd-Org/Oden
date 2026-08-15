FROM ghcr.io/cross-rs/aarch64-unknown-linux-gnu:main

RUN dpkg --add-architecture arm64 && \
    apt-get update && \
    apt-get install -y --no-install-recommends \
      libssl-dev:arm64 \
      libxcb-shape0-dev:arm64 libxcb-xfixes0-dev:arm64 \
      libxkbcommon-dev:arm64 libxkbcommon-x11-dev:arm64 libvulkan-dev:arm64 \
      libgl1-mesa-dev:arm64 libegl1-mesa-dev:arm64 \
      libfontconfig1-dev:arm64 libwayland-dev:arm64 && \
    rm -rf /var/lib/apt/lists/*

ENV PKG_CONFIG_ALLOW_CROSS=1 \
    PKG_CONFIG_PATH=/usr/lib/aarch64-linux-gnu/pkgconfig
