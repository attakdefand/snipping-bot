#!/usr/bin/env bash
set -euo pipefail
group="$1"
case "$group" in
 fmt) cargo fmt --all -- --check ;;
 clippy) cargo clippy --all-targets --all-features -- -D warnings ;;
 security) cargo install cargo-audit --locked --force || true; cargo audit ;;
 unit) cargo test --lib --workspace -- --nocapture ;;
 integration) cargo test -p sniper-storage -- --nocapture ;;
 e2e) ./ci/run-e2e.sh ;;
 *) echo "unknown group $group"; exit 2 ;;
esac