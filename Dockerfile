FROM rust:1.54.0-slim-buster AS builder
WORKDIR /app
RUN apt-get update \
 && apt-get install -y pkg-config libssl-dev \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml Cargo.lock ./
RUN mkdir src/ \
 && echo 'fn main() {}' > src/main.rs
RUN cargo build --locked \
 && rm -f target/release/deps/app*
COPY . .
RUN cargo test \
 && cargo install --path .

FROM debian:10.10-slim
RUN apt-get update \
 && apt-get install -y ca-certificates \
 && apt-get clean \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/post_random .
CMD /post_random
