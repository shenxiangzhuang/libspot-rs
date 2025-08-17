use std::env;
use std::path::PathBuf;

fn main() {
    // Get the path to the libspot directory
    let libspot_dir = PathBuf::from("../crates/libspot-ffi/libspot");
    let libspot_path = libspot_dir.join("dist/libspot.a.2.0b4");

    if !libspot_path.exists() {
        panic!("libspot library not found at {:?}. Please run 'make' in the libspot directory first.", libspot_path);
    }

    // Tell cargo to link with the static library
    println!("cargo:rustc-link-search=native={}", libspot_dir.join("dist").display());
    println!("cargo:rustc-link-lib=static=spot");
    
    // Tell cargo to invalidate the built crate whenever the library changes
    println!("cargo:rerun-if-changed={}", libspot_path.display());
}