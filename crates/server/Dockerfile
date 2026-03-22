# Build stage: compile the release binary.
FROM rust:1.90-slim@sha256:7fa728f3678acf5980d5db70960cf8491aff9411976789086676bdf0c19db39e AS builder
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
