use std::env;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let libspot_dir = Path::new(&manifest_dir).join("libspot");

    // Check if libspot library exists
    let lib_path = libspot_dir.join("dist").join("libspot.a.2.0b4");
    if !lib_path.exists() {
        panic!(
            "libspot library not found at {lib_path:?}. Please run 'make' in the libspot directory first."
        );
    }

    // Copy the pre-built library to OUT_DIR
    std::fs::copy(&lib_path, Path::new(&out_dir).join("libspot.a"))
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
