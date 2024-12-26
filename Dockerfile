FROM rust:1.83.0 AS builder
WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE=true
ENV APP_ENVIRONMENT=production
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl-dev
COPY --from=builder /app/target/release/actix-demo actix-demo
COPY --from=builder /app/configuration configuration
ENV APP_ENVIRONMENT=production
ENTRYPOINT ["./actix-demo"]
