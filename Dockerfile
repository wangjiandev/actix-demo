FROM rust:1.83.0
WORKDIR /app
RUN apt update && apt install lld clang -y
COPY . .
ENV SQLX_OFFLINE=true
ENV APP_ENVIRONMENT=production
RUN cargo build --release
ENTRYPOINT ["/app/target/release/actix-demo"]
