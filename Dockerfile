# 1. Use the official Rust image as a base builder environment
FROM rust:1.98 as builder

# 2. Add the official LLVM repository to get LLVM 22 assets natively
RUN apt-get update && apt-get install -y wget lsb-release software-properties-common gnupg \
    && wget https://llvm.org \
    && chmod +x llvm.sh \
    && ./llvm.sh 22 \
    && apt-get install -y llvm-22-dev libclang-22-dev clang-22 \
    && rm -rf /var/lib/apt/lists/*

# 3. Explicitly tell cargo where the LLVM 22 config file lives inside Linux
ENV LLVM_SYS_221_PREFIX=/usr/lib/llvm-22

# 4. Set up the working directory inside the container
WORKDIR /usr/src/poison_lang_playground

# 5. Copy the entire source code into the builder container
COPY . .

# 6. Build the optimized production binary
RUN cargo build --release --bin compiler_server

# 7. Create a minimal runtime image to save server memory
FROM debian:bookworm-slim

# 8. Install the corresponding shared runtime libraries for LLVM 22
RUN apt-get update && apt-get install -y wget gnupg lsb-release software-properties-common \
    && wget https://llvm.org \
    && chmod +x llvm.sh \
    && ./llvm.sh 22 \
    && apt-get install -y libllvm22 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# 9. Copy the compiled binary and the public frontend folder from the builder stage
COPY --from=builder /usr/src/poison_lang_playground/target/release/compiler_server .
COPY --from=builder /usr/src/poison_lang_playground/public ./public

# 10. Expose the standard Render port environment variable
EXPOSE 10000
ENV PORT=10000

# 11. Start the application binary directly
CMD ["./compiler_server"]
