FROM rust:1.47 AS planner
WORKDIR app
RUN cargo install cargo-chef --version 0.1.6
COPY . .
RUN cargo chef prepare --recipe-path recipe.json 

FROM rust:1.47 AS cacher
WORKDIR app
RUN cargo install cargo-chef --version 0.1.6
COPY --from=planner /app/recipe.json recipe.json
RUN RUST_BACKTRACE=1 cargo chef cook --release --recipe-path recipe.json

FROM rust:1.47 AS builder
WORKDIR app
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin zero2prod

FROM debian:buster-slim AS runtime
WORKDIR app
# OpenSSL is dynamically linked by some dependencies
RUN apt-get update -y \
    && apt-get install -y --no-install-recommends openssl \
    # Cleanup
    && apt-get autoremove -y \
    && apt-get clean -y \
    && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/zero2prod ./zero2prod
COPY configuration configuration
ENV APP_ENVIRONMENT production
ENTRYPOINT ["./zero2prod"]
