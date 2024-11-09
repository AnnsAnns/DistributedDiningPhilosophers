# Use the official Rust image from the Docker Hub
FROM lukemathwalker/cargo-chef:0.1.68-rust-1.82.0 AS chef
WORKDIR /app

# Set the working directory inside the container
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --recipe-path recipe.json

# Build application
COPY . .
RUN cargo build

# Copy built artifacts to a new stage
FROM rust:1.52.1-slim AS runtime

# Get OpenSSL
RUN apt-get update && apt-get install -y openssl

WORKDIR /app
COPY --from=builder /app/target/debug/ .