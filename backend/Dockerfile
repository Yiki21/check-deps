FROM rust:1.92 AS builder
ARG BUILD_PROFILE=release
ENV BUILD_PROFILE=${BUILD_PROFILE}

WORKDIR /app

# Fetch deps (uses full sources for path deps)
COPY . .
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/usr/local/cargo/git \
    if [ "$BUILD_PROFILE" = "release" ]; then PROFILE_FLAG="--release"; else PROFILE_FLAG=""; fi; \
    cargo build --workspace --bins $PROFILE_FLAG


FROM debian:stable-slim
ARG BUILD_PROFILE=release
ENV BUILD_PROFILE=${BUILD_PROFILE}
WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/${BUILD_PROFILE}/check-deps /app/check-deps
COPY --from=builder /app/target/${BUILD_PROFILE}/migrate /app/migrate
COPY application.yaml /app/application.yaml
COPY entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

ENV RUST_LOG=info
EXPOSE 8080

CMD ["/app/entrypoint.sh"]
