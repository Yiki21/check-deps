FROM rust:1.91 AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./
RUN mkdir -p src && \
    echo "fn main() { println!(\"dummy\"); }" > src/main.rs && \
    cargo build --release && \
    rm -rf src

COPY src ./src
RUN cargo build --release && \
    cp target/release/my_rust_app ./app

FROM scratch

COPY --from=builder /app/app /app

ENTRYPOINT ["/app"]
