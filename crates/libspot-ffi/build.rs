use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let libspot_dir = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("libspot");
    
    // Build libspot if not already built
    if !libspot_dir.join("dist").exists() {
        Command::new("make")
            .current_dir(&libspot_dir)
            .status()
            .expect("Failed to build libspot");
    }

    // Copy built library to OUT_DIR - try both versions
    let out_dir = env::var("OUT_DIR").unwrap();
    let library_paths = [
        libspot_dir.join("dist/libspot.a.2.0b4"),
        libspot_dir.join("dist/libspot.a.2.0b3"),
    ];
    
    let mut copied = false;
    for path in &library_paths {
        if path.exists() {
            fs::copy(path, Path::new(&out_dir).join("libspot.a"))
                .expect("Failed to copy library");
            copied = true;
            break;
        }
    }
    
    if !copied {
        panic!("Could not find libspot library at expected paths");
    }

    // Tell cargo where to find native libraries (search in OUT_DIR)
    println!("cargo:rustc-link-search=native={out_dir}");

    // Link against the static libspot library
    println!("cargo:rustc-link-lib=static=spot");

    // Link against the math library (required by libspot)
    println!("cargo:rustc-link-lib=m");

    // Rerun build script if libspot source files change
    println!("cargo:rerun-if-changed=libspot/");
}
