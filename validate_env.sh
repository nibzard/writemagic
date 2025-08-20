#!/bin/bash

echo "WriteMagic Rust Environment Validation"
echo "====================================="

# Set up environment
export RUST_LOG=info
cd /home/niko/writemagic

# Check basic Rust toolchain
echo "1. Checking Rust toolchain..."
rustup show || { echo "ERROR: rustup not working"; exit 1; }

echo -e "\n2. Checking cargo version..."
cargo --version || { echo "ERROR: cargo not working"; exit 1; }

echo -e "\n3. Installing toolchain if needed..."
rustup install 1.84.0
rustup default 1.84.0

echo -e "\n4. Checking workspace structure..."
cargo metadata --format-version 1 > /dev/null || { echo "ERROR: Invalid Cargo workspace"; exit 1; }

echo -e "\n5. Checking for compilation errors (shared domain)..."
cd core/shared
cargo check || { echo "ERROR: shared domain compilation failed"; exit 1; }

echo -e "\n6. Running shared domain tests..."
cargo test --lib || { echo "ERROR: shared domain tests failed"; exit 1; }

cd ../ai
echo -e "\n7. Checking AI domain compilation..."
cargo check || { echo "ERROR: AI domain compilation failed"; exit 1; }

echo -e "\n8. Running AI domain tests..."
cargo test --lib || { echo "ERROR: AI domain tests failed"; exit 1; }

cd ../writing
echo -e "\n9. Checking writing domain compilation..."
cargo check || { echo "ERROR: writing domain compilation failed"; exit 1; }

echo -e "\n10. Running writing domain tests..."
cargo test --lib || { echo "ERROR: writing domain tests failed"; exit 1; }

echo -e "\n✓ All environment validation checks passed!"
echo "WriteMagic Rust development environment is ready."