#!/bin/bash
set -e

echo "Testing Rust build environment for WriteMagic..."
echo "============================================="

echo "1. Checking Rust toolchain..."
rustup show

echo -e "\n2. Checking cargo version..."
cargo --version

echo -e "\n3. Running cargo check on workspace..."
cargo check --workspace

echo -e "\n4. Running tests on shared core..."
cargo test -p writemagic-shared

echo -e "\n5. Running tests on AI domain..."
cargo test -p writemagic-ai

echo -e "\n6. Running tests on writing domain..."
cargo test -p writemagic-writing

echo -e "\n7. Running tests on project domain..."
cargo test -p writemagic-project

echo -e "\nBuild environment test completed successfully!"