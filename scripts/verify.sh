#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
cargo fmt --all --check
cargo test --locked --no-default-features --features server --lib
cargo build --locked --bin fast-medium-api --no-default-features --features server
cargo check --locked --bin fast-medium --target wasm32-unknown-unknown --no-default-features --features web
python3 scripts/check_api.py
