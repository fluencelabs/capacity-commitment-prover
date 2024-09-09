FROM rustlang/rust:nightly-slim AS chef
USER root
RUN apt-get update && apt-get install -y clang llvm libhwloc-dev libudev-dev protobuf-compiler pkg-config cmake
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
WORKDIR /app
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS common-builder
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
      "linux/amd64") echo x86_64-unknown-linux-gnu > /rust_target.txt ;; \
      "linux/arm64") echo aarch64-unknown-linux-gnu > /rust_target.txt ;; \
      *) echo "Do $TARGETPLATFORM"; exit 1 ;; \
    esac ; \
    echo "Building for $TARGETPLATFORM via $(cat /rust_target.txt)"
WORKDIR /app
RUN rustup target add $(cat /rust_target.txt)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --target $(cat /rust_target.txt) --recipe-path recipe.json || true

FROM common-builder AS ccp-builder
WORKDIR /app
COPY . .
RUN cargo build --release --target $(cat /rust_target.txt) --bin ccp_cu
RUN cp /app/target/$(cat /rust_target.txt)/release/ccp_cu /tmp/ccp_cu

FROM ubuntu:latest AS user-builder
RUN useradd -u 10001 ccp_user && \
    apt update && \
    apt install -y libhwloc15 && \
    apt-get clean && \
    rm -rf \
        /tmp/* \
        /var/lib/apt/lists/* \
        /var/tmp/*

FROM user-builder AS ccp_main
COPY --from=ccp-builder /tmp/ccp_cu /usr/local/bin/ccp_cu
COPY cu_main/docker/Config.default.toml /Config.toml
COPY cu_main/docker/entrypoint.sh /entrypoint.sh
USER ccp_user
ENTRYPOINT ["/usr/local/bin/ccp_cu"]
