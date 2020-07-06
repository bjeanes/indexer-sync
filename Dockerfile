# -*- mode: dockerfile -*-

# You can override this `--build-arg BASE_IMAGE=...` to use different
# version of Rust or OpenSSL.
ARG BASE_IMAGE=ekidd/rust-musl-builder:latest

# Our first FROM statement declares the build environment.
FROM ${BASE_IMAGE} AS builder

# Build a cacheable layer with project dependencies
RUN USER=rust cargo new /home/rust/indexer-sync
WORKDIR /home/rust/indexer-sync
ADD --chown=rust:rust Cargo.* ./
RUN cargo build --release

# Add our source code.
ADD --chown=rust:rust . ./

# Build our application.
RUN touch src/main.rs # Docker's `ADD` does not bust cargo's build cache
RUN cargo build --release

# Now, we need to build our _real_ Docker container, copying in `bump-api`.
FROM alpine:latest
RUN apk --no-cache add ca-certificates

COPY --from=builder \
    /home/rust/indexer-sync/target/x86_64-unknown-linux-musl/release/indexer-sync \
    /usr/local/bin/

ENV RUST_LOG=warn,indexer_sync=info

CMD /usr/local/bin/indexer-sync
