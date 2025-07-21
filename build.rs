use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let libspot_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("libspot");
    let build_dir = Path::new(&out_dir).join("libspot");
    
    // Check if we're building for WASM
    let target = env::var("TARGET").unwrap_or_default();
    let is_wasm = target.contains("wasm32");
    
    if !is_wasm {
        // Native build process
        
        // Copy source and build in OUT_DIR
        Command::new("cp")
            .args([
                "-r",
                &libspot_dir.to_string_lossy(),
                &build_dir.to_string_lossy(),
            ])
            .status()
            .expect("Failed to copy libspot source");

        Command::new("make")
            .current_dir(&build_dir)
            .status()
            .expect("Failed to build libspot");

        // Copy built library to OUT_DIR
        fs::copy(
            build_dir.join("dist/libspot.a.2.0b3"),
            Path::new(&out_dir).join("libspot.a"),
        )
        .expect("Failed to copy library");

        // Tell cargo where to find native libraries (search in OUT_DIR)
        println!("cargo:rustc-link-search=native={out_dir}");

        // Link against the static libspot library
        println!("cargo:rustc-link-lib=static=spot");

        // Link against the math library (required by libspot)
        println!("cargo:rustc-link-lib=m");
    } else {
        // WASM build process
        println!("cargo:warning=Building for WASM target");
        
        // For WASM builds, we'll use the pre-built WASM module from libspot/wasm
        // No need to build the C library for WASM here
        
        // Tell cargo to rerun the build script if the WASM files change
        println!("cargo:rerun-if-changed=libspot/wasm/");
    }

    // Rerun build script if libspot source files change
    println!("cargo:rerun-if-changed=libspot/");
}
