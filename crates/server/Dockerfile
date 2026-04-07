# Build stage: compile the release binary.
# Digest pinned to rust:1.94-slim (multi-arch manifest list). Renovate keeps this current.
FROM rust:1.94-slim@sha256:48150eec4f1e854ec689c7ca4c16d35193c4ea19dde13b6df515192707229ff8 AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p merge_warden_server

# Runtime stage: minimal distroless image containing only the binary and its
# shared-library dependencies (libc/libgcc). No shell, no package manager.
FROM gcr.io/distroless/cc-debian12@sha256:329e54034ce498f9c6b345044e8f530c6691f99e94a92446f68c0adf9baa8464
COPY --from=builder /app/target/release/merge_warden_server /merge_warden_server
EXPOSE 3000
# The distroless runtime image contains no shell or HTTP client, so a
# Dockerfile-native HEALTHCHECK cannot run curl/wget. Container orchestrators
# (ECS, Azure Container Apps, Kubernetes) must configure an external HTTP
# liveness/readiness probe targeting GET /api/merge_warden on port 3000.
# This explicit NONE makes the intent clear rather than leaving it absent.
HEALTHCHECK NONE
ENTRYPOINT ["/merge_warden_server"]
