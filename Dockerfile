FROM rust:1.86

RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates curl && rm -rf /var/lib/apt/lists/*

WORKDIR /usr/src/app

COPY . .

RUN cargo build --release

CMD ["./target/release/axum-api-aws"]
