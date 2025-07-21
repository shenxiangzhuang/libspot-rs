use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let target = env::var("TARGET").unwrap();

    // Handle WASM targets
    if target.contains("wasm32") {
        println!("cargo:warning=WASM target detected.");

        let libspot_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("libspot");

        // Try to build the WASM library using the existing Makefile
        let status = Command::new("make")
            .current_dir(&libspot_dir)
            .arg("wasm/libspot.core.js")
            .status();

        match status {
            Ok(exit_status) => {
                if exit_status.success() {
                    println!("cargo:warning=Successfully built WASM library with emscripten");
                } else {
                    println!("cargo:warning=emscripten build failed, continuing without C library");
                }
            }
            Err(e) => {
                println!("cargo:warning=emscripten not available: {e}");
                println!(
                    "cargo:warning=Continuing without C library - FFI calls will fail at runtime"
                );
                println!("cargo:warning=To enable full WASM support, install emscripten:");
                println!("cargo:warning=  git clone https://github.com/emscripten-core/emsdk.git");
                println!(
                    "cargo:warning=  cd emsdk && ./emsdk install latest && ./emsdk activate latest"
                );
            }
        }

        // For WASM, we don't link against static libraries
        // The C code is compiled to WASM and loaded separately
        // We'll define the FFI functions as weak symbols so they don't cause link errors

        // Rerun build script if libspot source files change
        println!("cargo:rerun-if-changed=libspot/");
        return;
    }

    let out_dir = env::var("OUT_DIR").unwrap();
    let libspot_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("libspot");
    let build_dir = Path::new(&out_dir).join("libspot");

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

    // Rerun build script if libspot source files change
    println!("cargo:rerun-if-changed=libspot/");
}
