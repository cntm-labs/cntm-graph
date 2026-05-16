# Stage 1: Build Rust Core
FROM ubuntu:24.04 as builder
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y \
    curl build-essential libssl-dev pkg-config git \
    && rm -rf /var/lib/apt/lists/*

# Install Pixi
RUN curl -fsSL https://pixi.sh/install.sh | bash
ENV PATH="/root/.pixi/bin:${PATH}"

WORKDIR /app
COPY . .
# Build the Rust library and binaries
RUN cargo build --release

# Stage 2: Build Web Explorer
FROM node:20 as explorer-builder
WORKDIR /app/explorer
COPY explorer/package*.json ./
RUN npm install
COPY explorer/ .
RUN npm run build

# Stage 3: Runtime
FROM ubuntu:24.04
RUN apt-get update && apt-get install -y \
    ca-certificates python3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
# Copy Rust artifacts (assuming it produces a shared library and examples)
COPY --from=builder /app/target/release/cntm-graph .
COPY --from=builder /app/target/release/examples/stress_test_1b ./stress_test
COPY --from=explorer-builder /app/explorer/dist ./www

# Expose Web Explorer port
EXPOSE 5173

# Simple Python server to serve the Explorer UI
CMD ["python3", "-m", "http.server", "5173", "--directory", "www"]
