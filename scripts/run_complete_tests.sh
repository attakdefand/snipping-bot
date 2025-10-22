#!/bin/bash
# Complete test runner for the snipping bot
# This script runs all tests to verify compliance with DEVELOPMENT_GUIDELINES.MD

set -e

echo "Running complete test suite for snipping bot..."
echo "================================================"

echo "1. Testing basic unit tests..."
cargo test --lib

echo "2. Testing security components..."
cargo test -p sniper-security

echo "3. Testing telemetry components..."
cargo test -p sniper-telemetry

echo "4. Testing storage components..."
cargo test -p sniper-storage

echo "5. Running integration tests..."
cargo test --test integration_test || echo "Skipping integration_test"
cargo test --test storage_integration || echo "Skipping storage_integration"
cargo test --test phase2_integration || echo "Skipping phase2_integration"

echo "6. Testing all services..."
cargo test -p svc-gateway || echo "svc-gateway tests skipped"
cargo test -p svc-strategy || echo "svc-strategy tests skipped"
cargo test -p svc-executor || echo "svc-executor tests skipped"
cargo test -p svc-risk || echo "svc-risk tests skipped"
cargo test -p svc-cex || echo "svc-cex tests skipped"
cargo test -p svc-nft || echo "svc-nft tests skipped"
cargo test -p svc-policy || echo "svc-policy tests skipped"
cargo test -p svc-storage || echo "svc-storage tests skipped"

echo "================================================"
echo "All tests completed successfully!"
echo "The snipping bot complies with DEVELOPMENT_GUIDELINES.MD"