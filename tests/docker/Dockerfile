# syntax=docker/dockerfile:1

# ---- Stage 1: Build (Rust stable on Debian slim) ----
FROM rust:1-slim AS builder

WORKDIR /src

# Copy manifests first so cargo can resolve deps (layer cache friendly)
COPY Cargo.toml Cargo.lock rust-toolchain.toml ./

# Copy source and build release binary
COPY src/ ./src/
RUN cargo build --release --locked

# ---- Stage 2: Test environment (fresh Ubuntu, no fzf/tmux) ----
FROM ubuntu:24.04

ENV DEBIAN_FRONTEND=noninteractive

# Install only what's needed to run the test suite itself
# (git + sudo so Test 7 can install fzf/tmux via apt)
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        sudo \
        git \
    && rm -rf /var/lib/apt/lists/*

# Create an unprivileged user with passwordless sudo
# (simulates a regular developer who can install packages)
RUN useradd -m -s /bin/bash testuser \
    && echo "testuser ALL=(ALL) NOPASSWD:ALL" >> /etc/sudoers

# Install the tmuxido binary built in stage 1
COPY --from=builder /src/target/release/tmuxido /usr/local/bin/tmuxido

# Copy and register the test entrypoint
COPY tests/docker/entrypoint.sh /usr/local/bin/entrypoint.sh
RUN chmod +x /usr/local/bin/entrypoint.sh

USER testuser
WORKDIR /home/testuser

ENTRYPOINT ["entrypoint.sh"]
