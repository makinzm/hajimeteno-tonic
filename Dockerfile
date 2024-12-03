# Copy from https://github.com/makinzm/rust-wasm-github/blob/main/backend/Dockerfile
# ビルドステージ
FROM rust:1-slim-buster as builder

WORKDIR /usr/src/myapp

# 開発に必要なツールをインストール
RUN apt-get update && apt-get install -y \
    git \
    curl \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# ソースコードをマウントしてビルド
RUN --mount=type=bind,source=.,target=/usr/src/myapp \
    --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/src/myapp/target \
    cargo build --release && \
    cp target/release/hajimeteno-tonic /usr/local/bin/myapp

# 実行ステージ
FROM debian:buster-slim

# ビルドステージから実行可能ファイルをコピー
COPY --from=builder /usr/local/bin/myapp /bin/myapp

ENV SCYLLA_CONTACT_POINTS=scylladb
ENV SCYLLA_PORT=9042
ENV SCYLLA_KEYSPACE=vector_keyspace

CMD ["/bin/myapp"]
