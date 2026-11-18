# Stage 1: Build Nuxt SPA
FROM oven/bun:1 AS web
WORKDIR /app
COPY web/package.json web/bun.lock* ./
RUN bun install
COPY web/ .
RUN bunx --bun nuxt generate && test -f .output/public/index.html

# Stage 2: Build Rust binary
FROM rust:1-bookworm AS builder
# zpl-rs downloads the libzpl release matching its own version;
# LIBZPL_COPY_TO stages libzpl.so at a known path for the runtime stage.
ENV LIBZPL_COPY_TO=/app/lib
WORKDIR /app
COPY Cargo.toml Cargo.lock build.rs ./
# Cache dependencies (libzpl.so download happens here)
RUN mkdir src && echo 'fn main(){}' > src/main.rs && cargo build --release && rm src/main.rs
COPY src/ src/
RUN touch src/main.rs && cargo build --release

# Stage 3: Runtime (build.rs sets an $ORIGIN rpath, so libzpl.so sits next to the binary)
FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/zebra-emulator .
COPY --from=builder /app/lib/libzpl.so .
COPY --from=web /app/.output/public ./web
EXPOSE 8080
CMD ["./zebra-emulator", "-i", "0.0.0.0", "--web-dir", "./web"]
