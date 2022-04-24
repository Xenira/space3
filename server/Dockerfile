# Using the `rust-musl-builder` as base image, instead of 
# the official Rust toolchain
FROM ekidd/rust-musl-builder:latest AS chef
USER root
RUN cargo install cargo-chef
WORKDIR /app

FROM chef AS planner
COPY . .
COPY ../protocol /protocol
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
RUN apt install libpq-dev -y
COPY --from=planner /app/recipe.json recipe.json
COPY ../protocol /protocol
# Notice that we are specifying the --target flag!
RUN cargo chef cook --target x86_64-unknown-linux-musl --recipe-path recipe.json
COPY . .
RUN cargo build --target x86_64-unknown-linux-musl

FROM alpine
RUN addgroup -S serveruser && adduser -S serveruser -G serveruser
COPY --from=builder /app/target/x86_64-unknown-linux-musl/debug/server /usr/local/bin/
COPY ./static ./static
USER serveruser
EXPOSE 8000/tcp
CMD ["/usr/local/bin/server"]