# 1. Use the official Rust image as a base builder environment
FROM rust:1.98-bookworm as builder

# 2. Add the official LLVM repository explicitly and install LLVM 22
RUN apt-get update && apt-get install -y wget lsb-release software-properties-common gnupg \
    && wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | gpg --dearmor -o /etc/apt/trusted.gpg.d/apt.llvm.org.gpg \
    && echo "deb http://llvm.org llvm-toolchain-bookworm-22 main" >> /etc/apt/sources.list \
    && echo "deb-src http://llvm.org llvm-toolchain-bookworm-22 main" >> /etc/apt/sources.list \
    && apt-get update \
    && apt-get install -y llvm-22-dev libclang-22-dev clang-22 \
    && rm -rf /var/lib/apt/lists/*

# 3. Explicitly tell cargo where the LLVM 22 config files sit inside Linux
ENV LLVM_SYS_221_PREFIX=/usr/lib/llvm-22

WORKDIR /usr/src/poison_lang_playground
COPY . .

# 4. Build the optimized production server binary
RUN cargo build --release --bin compiler_server

# 5. Create a minimal execution runtime layer to save memory limits
FROM debian:bookworm-slim

# 6. Install the stable runtime system libraries matching the builder stage
RUN apt-get update && apt-get install -y wget gnupg lsb-release software-properties-common \
    && wget -qO- https://apt.llvm.org/llvm-snapshot.gpg.key | gpg --dearmor -o /etc/apt/trusted.gpg.d/apt.llvm.org.gpg \
    && echo "deb http://llvm.org llvm-toolchain-bookworm-22 main" >> /etc/apt/sources.list \
    && apt-get update \
    && apt-get install -y libllvm22 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 7. Move over nothing but the final binary asset and the frontend code folder
COPY --from=builder /usr/src/poison_lang_playground/target/release/compiler_server .
COPY --from=builder /usr/src/poison_lang_playground/public ./public

EXPOSE 10000
ENV PORT=10000

CMD ["./compiler_server"]
