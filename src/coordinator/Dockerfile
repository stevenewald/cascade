# Start with a Rust base image
FROM rust:1.68 AS builder

# Create a new empty shell project
WORKDIR /usr/src/app

# Copy over your Manifest file
COPY ./kubernetes/Cache.toml ./Cargo.toml

RUN mkdir src/ && echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && rm -rf src/

RUN rm Cargo.toml

# Copy your source code
COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
COPY ./build.rs ./build.rs
COPY ./.env ./.env
COPY ./proto ./proto

RUN apt-get update && apt-get install -y openssl protobuf-compiler

RUN cargo build --release

# Our Second stage, that creates the final image
FROM debian:buster-slim

# Install necessary libs
RUN apt-get update && apt-get install -y openssl ca-certificates

# Copy the build artifact from the builder stage
COPY --from=builder /usr/src/app/target/release/coordinator /usr/local/bin

# This is the command that will be run when the Docker image is run
CMD ["coordinator"]

# Expose ports
EXPOSE 50051

