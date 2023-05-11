#!/bin/bash

# Postgres library location (libpq)
RUSTFLAGS="-L native=/usr/local/opt/libpq/lib" cargo run