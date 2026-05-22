# Use cargo-chef for dependency caching
FROM lukemathwalker/cargo-chef:latest-rust-bookworm AS chef
WORKDIR /app

# Prepare the dependency recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Build stage
FROM chef AS builder

# Build dependencies
COPY --from=planner /app/recipe.json recipe.json
COPY --from=planner /app/migration /app/migration

RUN cargo chef cook --release --recipe-path recipe.json

# Install dioxus-cli (pin to match dioxus crate version in Cargo.toml)
RUN cargo install dioxus-cli --version 0.7.3 --locked

# Copy source and build
COPY . .
RUN dx bundle --platform web --release \
    && test -f target/dx/terrier/release/web/server \
    && test -d target/dx/terrier/release/web/public

# Runtime image
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Dioxus 0.7 bundles server binary and web assets under release/web/
COPY --from=builder /app/target/dx/terrier/release/web/server /app/terrier
COPY --from=builder /app/target/dx/terrier/release/web/public /app/public

EXPOSE 8080

CMD ["/app/terrier"]
