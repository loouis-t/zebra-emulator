# Stage 1: Build Nuxt SPA
FROM oven/bun:1 AS web
WORKDIR /app
COPY web/package.json web/bun.lock* ./
RUN bun install
COPY web/ .
RUN bunx --bun nuxt generate && test -f .output/public/index.html

# Stage 2: Build Rust binary
FROM rust:1-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
# Cache dependencies (libzpl.so download happens here)
RUN mkdir src && echo 'fn main(){}' > src/main.rs && cargo build --release && rm src/main.rs
COPY src/ src/
RUN touch src/main.rs && cargo build --release
# Move libzpl.so to a known path
RUN find target/release/build -name 'libzpl.so' -exec cp {} target/release/ \;

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/zebra-emulator .
COPY --from=builder /app/target/release/libzpl.so .
COPY --from=web /app/.output/public ./web
ENV LD_LIBRARY_PATH=/app
EXPOSE 8080
CMD ["./zebra-emulator", "-i", "0.0.0.0", "--web-dir", "./web"]
