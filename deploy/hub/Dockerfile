# quire's sync hub (`.knowledge/sync-hub.md`): one image for self-hosting and for our cloud.
# Build from the workspace root:  podman build -f deploy/hub/Dockerfile -t quire-hub .
#
# Two stages: the Rust build, then only the binary on a small pinned base (distroless, glibc, no
# shell, non-root). SQLite is built into the binary (rusqlite "bundled"); TLS is rustls.

FROM docker.io/library/rust:1.99.0-slim-trixie AS build
WORKDIR /src
COPY . .
RUN cargo build --release --locked -p quire-hub \
 && mkdir -p /out/data \
 && cp target/release/quire-hub /out/quire-hub

FROM gcr.io/distroless/cc-debian13:nonroot
COPY --from=build /out/quire-hub /usr/local/bin/quire-hub
# The data folder belongs to the image's non-root user (65532), so a new volume does too.
COPY --from=build --chown=65532:65532 /out/data /data
ENV QUIRE_HUB_DATA=/data
VOLUME ["/data"]
USER nonroot
ENTRYPOINT ["/usr/local/bin/quire-hub"]
CMD ["serve"]
HEALTHCHECK --interval=30s --timeout=5s --start-period=30s CMD ["/usr/local/bin/quire-hub", "health"]
