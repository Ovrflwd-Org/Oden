# Oden


[![Test](https://github.com/Ovrflwd-Org/Oden/actions/workflows/ci.yml/badge.svg)](https://github.com/Ovrflwd-Org/Oden/actions/workflows/ci.yml) [![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org/) 

Your commands, snippets, and notes live in your head. Oden puts them somewhere better. Store everything in one place, link related pieces together, and navigate your own knowledge graph, so you stop Googling the same thing twice.

## Installation

Every [release](https://github.com/out-of-order/oden/releases) from 0.8.0+ publishes binaries for:

- **Windows** (x64, arm64) ; `oden-win-<arch>_<version>.zip` (portable, used by the in-app updater), or `oden-win-<arch>_<version>.exe`, an NSIS installer with Start Menu shortcuts and a proper uninstaller
- **macOS** (Apple Silicon only) ; `oden-macos-arm64_<version>.dmg` or `.pkg`
- **Linux** (x64, arm64) ; `oden-linux-<arch>_<version>.AppImage`, `.flatpak`, `.snap`, `.deb`, `.rpm`, and Arch (`.pkg.tar.zst`)

Oden checks for updates on startup (configurable in Settings) and, on the portable channels (Windows zip or NSIS install, AppImage, unpackaged macOS), can update itself in place ; the NSIS installer puts Oden in a per-user, writable location, so it self-updates the same way the portable zip does. Installs made through a real package manager (Flatpak, Snap, deb, rpm, Arch) are updated the normal way, through that package manager.

Flatpak and Snap builds are currently distributed as downloadable files from the GitHub release rather than published to Flathub/the Snap Store until further notice.

## Development

- [Testing the in-app updater](docs/fake-update-server.md) ; run a local mock update server to exercise the check/install/restart flow and the "what's new" changelog popup without touching the real release channel.
