# syntax=docker/dockerfile:1
# Runtime image using pre-compiled binaries.
#
# Rust binaries for each supported architecture are compiled natively in CI
# (x86_64 via cargo, aarch64 via cross) and staged under
# binaries/<TARGETARCH>/ in the build context before this image is built.
#
# Docker Buildx sets TARGETARCH automatically during multi-platform builds
# (e.g. 'amd64', 'arm64'), selecting the correct pre-compiled binary.
#
# Debian 13 (Trixie) is required because the CI builder uses Debian 13 tooling
# which ships glibc 2.38. The older distroless/cc-debian12 only provides
# glibc 2.36 and causes a "GLIBC_2.38 not found" crash at startup.
FROM gcr.io/distroless/cc-debian13@sha256:a017e74bd2a12d98342dbecd33d121d2b160415ed777573dc1808969e989d94d
ARG TARGETARCH
COPY binaries/${TARGETARCH}/merge_warden_server /merge_warden_server
EXPOSE 3000
# The distroless runtime image contains no shell or HTTP client, so a
# Dockerfile-native HEALTHCHECK cannot run curl/wget. Container orchestrators
# (ECS, Azure Container Apps, Kubernetes) must configure an external HTTP
# liveness/readiness probe targeting GET /health on port 3000.
# This explicit NONE makes the intent clear rather than leaving it absent.
HEALTHCHECK NONE
ENTRYPOINT ["/merge_warden_server"]
