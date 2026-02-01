# Multi-stage build for MuPDF MCP Server

# Build stage
FROM rust:1.85-bookworm AS builder

# Install MuPDF build dependencies
RUN apt-get update && apt-get install -y \
    build-essential \
    pkg-config \
    libfreetype6-dev \
    libharfbuzz-dev \
    libjpeg-dev \
    libopenjp2-7-dev \
    zlib1g-dev \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy source files
COPY Cargo.toml Cargo.lock* ./
COPY src ./src

# Build the project
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libfreetype6 \
    libharfbuzz0b \
    libjpeg62-turbo \
    libopenjp2-7 \
    zlib1g \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the built binary
COPY --from=builder /app/target/release/mupdf-mcp-server /app/mupdf-mcp-server

# Set the entrypoint
ENTRYPOINT ["/app/mupdf-mcp-server"]
