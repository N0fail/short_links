FROM rust:1.79
COPY src/ /app/src
COPY Cargo.toml /app
WORKDIR /app
EXPOSE 80
CMD ["cargo", "run", "--release"]
