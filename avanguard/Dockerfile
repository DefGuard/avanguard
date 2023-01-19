FROM lukemathwalker/cargo-chef:0.1.51-rust-1.66 AS chef
WORKDIR /build

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
ENV SQLX_OFFLINE true
RUN cargo build --release --bin avanguard

FROM debian:bullseye-slim AS runtime
COPY --from=builder /build/target/release/avanguard /usr/local/bin
ENTRYPOINT ["/usr/local/bin/avanguard"]
