#!/bin/bash

docker build -t space3:wasm . && docker run --rm -it -e RUST_BACKTRACE=1 -e DATABASE_URL='postgres://postgres:mysecretpassword@postgres/postgres' -e ROCKET_LOG_LEVEL=debug -e ROCKET_ADDRESS=0.0.0.0 -p 8000:8000 --link postgres space3:wasm
