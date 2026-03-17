# Build stage: compile the release binary.
FROM rust:1.90-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build --release -p merge_warden_server

# Runtime stage: minimal distroless image containing only the binary and its
# shared-library dependencies (libc/libgcc). No shell, no package manager.
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/merge_warden_server /merge_warden_server
EXPOSE 3000
# The distroless runtime image contains no shell or HTTP client, so a
# Dockerfile-native HEALTHCHECK cannot run curl/wget. Container orchestrators
# (ECS, Azure Container Apps, Kubernetes) must configure an external HTTP
# liveness/readiness probe targeting GET /api/merge_warden on port 3000.
# This explicit NONE makes the intent clear rather than leaving it absent.
HEALTHCHECK NONE
ENTRYPOINT ["/merge_warden_server"]
