FROM rust:1.54.0-slim-buster AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
RUN mkdir src/ \
  && echo "fn main() {}" > src/main.rs
RUN cargo build --locked \
  && rm -f target/release/deps/app*
COPY . .
RUN cargo test \
  && cargo install --path .

FROM debian:10.10-slim
COPY --from=builder /usr/local/cargo/bin/post_random .
CMD /post_random
