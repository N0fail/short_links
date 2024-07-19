FROM rust:1.79

# Create a new empty shell project
RUN USER=root cargo new --bin short_links
WORKDIR /short_links

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Build only the dependencies to cache them
RUN cargo build --release
RUN rm src/*.rs

# Copy the source code
COPY src ./src

# Build the application
RUN cargo build --release

# Expose the port that the application will run on
EXPOSE 80

# Set the startup command
CMD ["cargo", "run", "--release"]
