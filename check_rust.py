#!/usr/bin/env python3
import subprocess
import os
import sys

def run_command(cmd, description):
    print(f"Running: {description}")
    print(f"Command: {' '.join(cmd)}")
    try:
        result = subprocess.run(cmd, capture_output=True, text=True, cwd="/home/niko/writemagic")
        print(f"Return code: {result.returncode}")
        if result.stdout:
            print(f"STDOUT:\n{result.stdout}")
        if result.stderr:
            print(f"STDERR:\n{result.stderr}")
        print("-" * 50)
        return result.returncode == 0
    except Exception as e:
        print(f"Error running command: {e}")
        print("-" * 50)
        return False

def main():
    print("WriteMagic Rust Environment Check")
    print("=" * 40)
    
    # Check if we're in the right directory
    print(f"Current working directory: {os.getcwd()}")
    print(f"Target directory: /home/niko/writemagic")
    print("-" * 50)
    
    commands = [
        (["rustup", "show"], "Check Rust toolchain"),
        (["cargo", "--version"], "Check Cargo version"),
        (["cargo", "check", "--workspace"], "Cargo check workspace"),
        (["cargo", "test", "-p", "writemagic-shared", "--lib"], "Test shared domain"),
        (["cargo", "test", "-p", "writemagic-ai", "--lib"], "Test AI domain"),
    ]
    
    results = []
    for cmd, desc in commands:
        success = run_command(cmd, desc)
        results.append((desc, success))
    
    print("\nSUMMARY:")
    print("=" * 40)
    for desc, success in results:
        status = "✓ PASS" if success else "✗ FAIL"
        print(f"{status}: {desc}")

if __name__ == "__main__":
    main()