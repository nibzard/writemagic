//! Build script for WriteMagic WASM crate
//! 
//! This build script handles:
//! - WASM-specific environment setup
//! - Feature flag validation
//! - Conditional compilation setup

use std::env;

fn main() {
    // Tell Cargo to re-run this build script if the environment changes
    println!("cargo:rerun-if-env-changed=WASM_TARGET");
    println!("cargo:rerun-if-env-changed=WASM_OPT_LEVEL");
    
    // Check if we're building for WASM target
    let target = env::var("TARGET").unwrap_or_default();
    if target.starts_with("wasm32") {
        println!("cargo:rustc-cfg=wasm_target");
        
        // Enable WASM-specific features
        println!("cargo:rustc-cfg=feature=\"wasm\"");
        
        // Set optimization hints for WASM
        if let Ok(opt_level) = env::var("WASM_OPT_LEVEL") {
            println!("cargo:rustc-env=WASM_OPT_LEVEL={}", opt_level);
        }
        
        // Configure WASM-specific linker flags
        println!("cargo:rustc-link-arg=--max-memory=67108864"); // 64MB max memory
        
        // Enable SIMD if available
        if env::var("CARGO_CFG_TARGET_FEATURE").unwrap_or_default().contains("simd128") {
            println!("cargo:rustc-cfg=wasm_simd");
        }
    }
    
    // Validate feature combinations
    let console_hook = env::var("CARGO_FEATURE_CONSOLE_ERROR_PANIC_HOOK").is_ok();
    if console_hook {
        println!("cargo:rustc-cfg=console_panic_hook");
    }
    
    println!("cargo:rustc-env=BUILD_TIMESTAMP={}", chrono::Utc::now().timestamp());
}