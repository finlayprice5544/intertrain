# Railway build for a generated site (Rust backend + cloned frontend).
# Build context = the project root (output/<name>/) containing backend/ and frontend/.
# syntax=docker/dockerfile:1

# ---- build stage ----
FROM rust:1-bookworm AS build
# rocksdb/bindgen -> clang+libclang ; native-tls -> openssl
RUN apt-get update && apt-get install -y --no-install-recommends \
      clang libclang-dev pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY backend/ ./backend/
WORKDIR /app/backend
RUN cargo build --release --bin generated-site

# ---- runtime stage ----
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
      ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=build /app/backend/target/release/generated-site /usr/local/bin/generated-site
# runtime assets (read relative to /app)
COPY backend/config ./config
COPY backend/seed ./seed
COPY backend/migrations ./migrations
COPY frontend ./frontend
ENV PUBLIC_DIR=/app/frontend \
    ROCKSDB_PATH=/data/rocks \
    DATA_MODE=auto \
    RUST_LOG=info
# Railway injects $PORT at runtime; the app reads it (default 8787).
EXPOSE 8080
CMD ["generated-site"]
