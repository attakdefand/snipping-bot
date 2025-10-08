#!/usr/bin/env bash
set -euo pipefail
export RUST_LOG=${RUST_LOG:-info}
# run a few services locally using the in-memory bus
cargo run -p svc-signals >/tmp/svc-signals.log 2>&1 &
SIG=$!
cargo run -p svc-strategy >/tmp/svc-strategy.log 2>&1 &
STR=$!
cargo run -p svc-executor >/tmp/svc-executor.log 2>&1 &
EXE=$!
cargo run -p svc-gateway
kill $SIG $STR $EXE || true
