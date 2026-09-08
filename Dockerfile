FROM rust:1.98 as builder

RUN apt-get update && apt-get install -y wget lsb-release software-properties-common gnupg \
    && wget https://llvm.org \
    && chmod +x llvm.sh \
    && ./llvm.sh 22 \
    && apt-get install -y llvm-22-dev libclang-22-dev clang-22 \
    && rm -rf /var/lib/apt/lists/*

ENV LLVM_SYS_221_PREFIX=/usr/lib/llvm-22

WORKDIR /usr/src/poison_lang_playground

COPY . .

RUN cargo build --release --bin compiler_server

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y wget gnupg lsb-release software-properties-common \
    && wget https://llvm.org \
    && chmod +x llvm.sh \
    && ./llvm.sh 22 \
    && apt-get install -y libllvm22 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /usr/src/poison_lang_playground/target/release/compiler_server .
COPY --from=builder /usr/src/poison_lang_playground/public ./public

EXPOSE 10000
ENV PORT=10000

CMD ["./compiler_server"]

