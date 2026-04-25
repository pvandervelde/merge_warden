# syntax=docker/dockerfile:1
# Build stage: compile the release binary.
# Digest pinned to rust:1.94-slim (multi-arch manifest list). Renovate keeps this current.
FROM rust:1.94-slim@sha256:48150eec4f1e854ec689c7ca4c16d35193c4ea19dde13b6df515192707229ff8 AS builder
WORKDIR /app
RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev zlib1g-dev libzstd-dev && \
    rm -rf /var/lib/apt/lists/*
# Force static linking of OpenSSL so the runtime image does not need libssl.so.
# The distroless/cc runtime image provides glibc/libgcc but not OpenSSL shared
# libraries, so a dynamically-linked binary would crash at startup.
ENV OPENSSL_STATIC=1
COPY . .
# Cache the cargo registry (downloaded crates) and the build target directory
# (compiled artifacts) across Docker builds so only changed crates are recompiled.
# The binary is copied out of the cache mount before the stage ends so it is
# accessible in the runtime stage.
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/app/target \
    cargo build --release -p merge_warden_server && \
    cp /app/target/release/merge_warden_server /merge_warden_server_bin

# Runtime stage: minimal distroless image containing only the binary and its
# shared-library dependencies (libc/libgcc). No shell, no package manager.
# Debian 13 (Trixie) is required because rust:1.94-slim also uses Debian 13,
# which ships glibc 2.38. The older distroless/cc-debian12 only provides
# glibc 2.36 and causes a "GLIBC_2.38 not found" crash at startup.
FROM gcr.io/distroless/cc-debian13@sha256:56aaf20ab2523a346a67c8e8f8e8dabe447447d0788b82284d14ad79cd5f93cc
COPY --from=builder /merge_warden_server_bin /merge_warden_server
EXPOSE 3000
# The distroless runtime image contains no shell or HTTP client, so a
# Dockerfile-native HEALTHCHECK cannot run curl/wget. Container orchestrators
# (ECS, Azure Container Apps, Kubernetes) must configure an external HTTP
# liveness/readiness probe targeting GET /api/merge_warden on port 3000.
# This explicit NONE makes the intent clear rather than leaving it absent.
HEALTHCHECK NONE
ENTRYPOINT ["/merge_warden_server"]
