##################################################
# General
##################################################
FROM rust:alpine AS chef
USER root
RUN apk update && apk add --no-cache openssl-dev musl-dev libpq-dev && cargo install cargo-chef
WORKDIR /app

##################################################
# Client
##################################################
FROM chef AS trunk
WORKDIR /client
RUN cargo install --locked trunk
RUN rustup target add wasm32-unknown-unknown

FROM chef AS planner-client
COPY ./client /client
COPY ./protocol /protocol
WORKDIR /client
RUN cargo chef prepare --recipe-path recipe.json

FROM trunk AS builder-client
COPY --from=planner-client /client/recipe.json /client/recipe.json
COPY ./protocol /protocol
RUN cargo chef cook --target wasm32-unknown-unknown --recipe-path recipe.json
COPY ./client /client
RUN trunk build

##################################################
# Server
##################################################

FROM chef AS builder-server
COPY ./protocol /protocol
COPY ./server /server
RUN cd /server && RUSTFLAGS="-C target-feature=-crt-static" cargo build --target x86_64-unknown-linux-musl --release

##################################################
# Final Image
##################################################
FROM chef as server
WORKDIR /usr/local/bin
RUN apk add libgcc && addgroup -S serveruser && adduser -S serveruser -G serveruser
COPY --from=builder-server /server/target/x86_64-unknown-linux-musl/release/rog-server .
COPY --from=builder-client /client/dist ./static
COPY --from=builder-client /client/assets ./static/assets
COPY ./server/static ./static
USER root
EXPOSE 8000/tcp
CMD ["rog-server"]
