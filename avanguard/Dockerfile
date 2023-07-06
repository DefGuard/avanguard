FROM lukemathwalker/cargo-chef:0.1.61-rust-1.70 AS chef
WORKDIR /build

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder 
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json
COPY . .
RUN cargo install --locked --path . --root /output

FROM debian:bullseye-slim AS runtime
COPY --from=builder /output/bin/avanguard /usr/local/bin
ENTRYPOINT ["/usr/local/bin/avanguard"]
