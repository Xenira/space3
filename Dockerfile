##################################################
# General
##################################################
FROM ekidd/rust-musl-builder:latest AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

##################################################
# Client
##################################################
FROM chef AS trunk
RUN cargo install --locked trunk
RUN rustup target add wasm32-unknown-unknown

FROM chef AS planner-client
COPY ./client .
COPY ./protocol /protocol
COPY ./bevy_forms /bevy_forms
RUN cargo chef prepare --recipe-path recipe.json

FROM trunk AS builder-client
COPY --from=planner-client /app/recipe.json recipe.json
COPY ./protocol /protocol
COPY ./bevy_forms /bevy_forms
RUN cargo chef cook --target wasm32-unknown-unknown --recipe-path recipe.json
COPY ./client .
RUN ~/.cargo/bin/trunk build

##################################################
# Server
##################################################
# Using the `rust-musl-builder` as base image, instead of 
# the official Rust toolchain

FROM chef AS planner-server
COPY ./server .
COPY ./protocol /protocol
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder-server
RUN apt install libpq-dev -y
COPY --from=planner-server /app/recipe.json recipe.json
COPY ./protocol /protocol
RUN cargo chef cook --target x86_64-unknown-linux-musl --release --recipe-path recipe.json
COPY ./server .
RUN cargo build --target x86_64-unknown-linux-musl --release

##################################################
# Final Image
##################################################
FROM alpine as server
WORKDIR /usr/local/bin
RUN addgroup -S serveruser && adduser -S serveruser -G serveruser
COPY --from=builder-server /app/target/x86_64-unknown-linux-musl/release/server .
COPY --from=builder-client /app/dist ./static
COPY ./server/static ./static
COPY ./client/assets ./static/assets
USER serveruser
EXPOSE 8000/tcp
CMD ["server"]