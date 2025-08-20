#!/bin/bash
set -e

echo "=== WriteMagic Rust Environment Test ==="
echo "Testing the fixes applied to the Rust development environment"
echo ""

# Change to project root
cd /home/niko/writemagic

echo "1. Checking Rust toolchain..."
rustup show

echo -e "\n2. Verifying toolchain version..."
rustc --version

echo -e "\n3. Checking workspace metadata..."
cargo metadata --format-version 1 --offline > /dev/null && echo "✅ Workspace structure valid"

echo -e "\n4. Testing shared domain compilation..."
cd core/shared
cargo check && echo "✅ Shared domain compiles"

echo -e "\n5. Testing shared domain tests..."
cargo test --lib --quiet && echo "✅ Shared domain tests pass"

echo -e "\n6. Testing AI domain compilation..."
cd ../ai
cargo check && echo "✅ AI domain compiles"

echo -e "\n7. Testing AI domain tests..."
cargo test --lib --quiet && echo "✅ AI domain tests pass"

echo -e "\n8. Testing writing domain compilation..."
cd ../writing
cargo check && echo "✅ Writing domain compiles"

echo -e "\n9. Testing writing domain tests..."
cargo test --lib --quiet && echo "✅ Writing domain tests pass"

echo -e "\n10. Testing project domain compilation..."
cd ../project
cargo check && echo "✅ Project domain compiles"

echo -e "\n11. Testing project domain tests..."
cargo test --lib --quiet && echo "✅ Project domain tests pass"

echo -e "\n12. Testing full workspace..."
cd /home/niko/writemagic
cargo check --workspace && echo "✅ Full workspace compiles"

echo -e "\n🎉 SUCCESS: All environment tests passed!"
echo ""
echo "WriteMagic Rust development environment is now ready for development."
echo ""
echo "Next steps:"
echo "- Run 'cargo test --workspace' to execute all tests"
echo "- Start developing core domain functionality"
echo "- Begin Android FFI integration"
echo "- Implement WASM compilation for web support"