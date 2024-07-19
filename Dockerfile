FROM rust:1.79
COPY src/ /app/src
COPY Cargo.toml /app
WORKDIR /app
CMD ["cargo", "run", "--release"]
