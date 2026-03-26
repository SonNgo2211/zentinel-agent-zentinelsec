# syntax=docker/dockerfile:1.4

# Zentinel ZentinelSec Agent Container Image
#
# Targets:
#   - prebuilt: For CI with pre-built binaries

################################################################################
# Pre-built binary stage (for CI builds)
################################################################################
# Build arguments
ARG RUST_VERSION=1.88
ARG DEBIAN_VARIANT=slim-bookworm

################################################################################
# Build stage - compiles the Rust binary with optimizations
################################################################################
FROM rust:${RUST_VERSION}-${DEBIAN_VARIANT} AS builder

# Install build dependencies (only what's needed for compilation)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
        pkg-config \
        libssl-dev \
        protobuf-compiler \
        cmake \
        build-essential \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifest files first for better layer caching
COPY zentinelsec-agent/Cargo.toml zentinelsec-agent/
COPY zentinel-modsec/zentinel-modsec-main /app/zentinel-modsec/zentinel-modsec-main
COPY zentinelsec-agent/src zentinelsec-agent/src

WORKDIR /app/zentinelsec-agent
# Build dependencies and agent
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12:nonroot

COPY --from=builder /app/zentinelsec-agent/target/release/zentinel-zentinelsec-agent /zentinel-zentinelsec-agent

LABEL org.opencontainers.image.title="Zentinel ZentinelSec Agent" \
      org.opencontainers.image.description="Zentinel ZentinelSec Agent for Zentinel reverse proxy" \
      org.opencontainers.image.vendor="Raskell" \
      org.opencontainers.image.source="https://github.com/zentinelproxy/zentinel-agent-zentinelsec"

ENV RUST_LOG=info,zentinel_zentinelsec_agent=debug \
    SOCKET_PATH=/var/run/zentinel/zentinelsec.sock

USER nonroot:nonroot

ENTRYPOINT ["/zentinel-zentinelsec-agent"]
