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
ENTRYPOINT ["/merge_warden_server"]
