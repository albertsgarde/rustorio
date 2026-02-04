FROM lukemathwalker/cargo-chef:0.1.73-rust-1.93 AS chef
WORKDIR /app
RUN cargo install dioxus-cli --locked

FROM chef AS planner
COPY rustorio-website/ .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

COPY rustorio-website/rust-toolchain.toml .


COPY rustorio-website/ .


# Create the final bundle folder. Bundle with release build profile to enable optimizations.
RUN dx bundle --web --release --package rustorio-website

FROM debian:trixie-slim AS runtime

WORKDIR /app
COPY --from=builder /app/target/dx/rustorio-website/release/web/ ./

# set our port and make sure to listen for all connections
ENV PORT=8080
ENV IP=0.0.0.0

# expose the port 8080
EXPOSE 8080
ENTRYPOINT [ "/app/rustorio-website", "--db-path", "/app/data/rustorio.db" ]
