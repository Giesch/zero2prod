FROM rust:1.50 AS builder
WORKDIR app
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin zero2prod
RUN mv /app/target/release/zero2prod . && rm -rf target

FROM debian:buster-slim AS runtime
WORKDIR app
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/zero2prod zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]
