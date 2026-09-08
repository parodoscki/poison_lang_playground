# 1. Use the official Rust image as a base builder environment
FROM rust:1.98 as builder

RUN apt-get update && apt-get install -y \
    llvm-dev \
    libclang-dev \
    clang \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/poison_lang_playground

COPY . .

RUN cargo build --release --bin compiler_server

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libllvm15 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/poison_lang_playground/target/release/compiler_server .
COPY --from=builder /usr/src/poison_lang_playground/public ./public

EXPOSE 10000
ENV PORT=10000

CMD ["./compiler_server"]
