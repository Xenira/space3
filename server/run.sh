#!/bin/bash

# Postgres library location (libpq)
#RUST_LOG=debug RUSTFLAGS="-L native=/usr/local/opt/libpq/lib" cargo run

docker build -t space3:dev . && docker run --rm -it -e DATABASES='postgres://postgres:mysecretpassword@postgres/postgres' -e ROCKET_LOG_LEVEL=debug -e ROCKET_ADDRESS=0.0.0.0 -p 8000:8000 --link postgres space3:dev