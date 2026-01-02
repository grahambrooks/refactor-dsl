# syntax=docker/dockerfile:1
ARG VARIANT="bookworm"
FROM mcr.microsoft.com/devcontainers/base:${VARIANT}

# Install Rust toolchain via rustup
ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    PATH=/usr/local/cargo/bin:${PATH}

RUN apt-get update && apt-get install -y --no-install-recommends \
        build-essential \
        pkg-config \
        libssl-dev \
        git \
    && rm -rf /var/lib/apt/lists/* \
    && curl https://sh.rustup.rs -sSf | sh -s -- -y --default-toolchain stable \
    && rustup component add clippy rustfmt \
    && chown -R 1000:1000 /usr/local/cargo /usr/local/rustup

# Pre-create cargo dirs and make them group-writable
RUN mkdir -p /usr/local/cargo/registry /usr/local/cargo/git \
    && chmod -R 0777 /usr/local/cargo

USER vscode

WORKDIR /workspaces/refactor-dsl

# Pre-cache dependencies (optional; uses Docker build cache)
COPY Cargo.toml Cargo.lock ./
RUN cargo fetch || true

# Copy source last to leverage layer caching
COPY . .

