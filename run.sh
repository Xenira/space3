#!/bin/bash

docker build -t space3:wasm --build-arg BASE_URL="http://localhost:8000" . && docker run --rm -it -e RUST_BACKTRACE=1 -e DATABASE_URL='postgres://postgres:mysecretpassword@postgres/postgres' -e ROCKET_LOG_LEVEL=debug -e ROCKET_ADDRESS=0.0.0.0 -p 8000:8000 --link postgres --net postgres space3:wasm
