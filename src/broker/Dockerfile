FROM rust:1.68 AS builder

WORKDIR /usr/src/app

COPY ./kubernetes/Cache.toml ./Cargo.toml

# Cache dependencies
RUN mkdir src/ && echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs && \
    cargo build --release && rm -rf src/

RUN rm Cargo.toml

COPY ./Cargo.toml ./Cargo.toml
COPY ./src ./src
COPY ./build.rs ./build.rs
COPY ./proto ./proto

RUN apt-get update && apt-get install -y openssl protobuf-compiler

RUN cargo build --release

# creates actual image
# earlier steps cache dependencies so we don't have to pull them every time we modify files
FROM debian:buster-slim

RUN apt-get update && apt-get install -y openssl ca-certificates

COPY --from=builder /usr/src/app/target/release/broker /usr/local/bin

# only difference in consumer/coordinator deployments is the end binary run
CMD ["broker"]

EXPOSE 50050 50051

