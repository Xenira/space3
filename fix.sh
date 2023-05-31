#!/bin/bash
set -e

cd client
cargo update && cargo fix --allow-dirty --allow-staged && cargo fmt
cd ../server
cargo update && cargo fix --allow-dirty --allow-staged && cargo fmt
cd ../protocol
cargo update && cargo fix --allow-dirty --allow-staged && cargo fmt
cd protocol_types
cargo update && cargo fix --allow-dirty --allow-staged && cargo fmt
cd ../protocol_data_types
cargo update && cargo fix --allow-dirty --allow-staged && cargo fmt
cd ../../cli
cargo update && cargo fix --allow-dirty --allow-staged && cargo fmt