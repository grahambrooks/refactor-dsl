#!/usr/bin/env bash
set -euo pipefail

# Ensure cargo registry and git dirs exist and are writable for the dev user
mkdir -p /usr/local/cargo/registry /usr/local/cargo/git
sudo chown -R $(id -u):$(id -g) /usr/local/cargo /usr/local/rustup || true
chmod -R 0777 /usr/local/cargo || true

# Print diagnostics
ls -ld /usr/local/cargo /usr/local/cargo/registry /usr/local/cargo/git

# Verify cargo works
cargo --version
rustc --version

