FROM rust:1.79
COPY src/ /app/src
COPY Cargo.toml /app
WORKDIR /app
RUN cargo build --release
ENTRYPOINT ./target/release/short_links
