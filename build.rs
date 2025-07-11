use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let out_dir = env::var("OUT_DIR").unwrap();

    // Path to the libspot directory
    let libspot_dir = Path::new(&manifest_dir).join("libspot");
    let build_dir = Path::new(&out_dir).join("libspot_build");

    // Create build directory in OUT_DIR
    fs::create_dir_all(&build_dir).expect("Failed to create build directory");

    // Copy libspot source to build directory using cp command
    let cp_output = Command::new("cp")
        .args([
            "-r",
            libspot_dir.to_str().unwrap(),
            build_dir.to_str().unwrap(),
        ])
        .output()
        .expect("Failed to copy libspot source. Make sure cp is available.");

    if !cp_output.status.success() {
        panic!(
            "Failed to copy libspot source: {}",
            String::from_utf8_lossy(&cp_output.stderr)
        );
    }

    // The cp command copies to build_dir/libspot, so adjust the path
    let actual_build_dir = build_dir.join("libspot");
    let build_dist_dir = actual_build_dir.join("dist");

    // Build libspot using make in the build directory
    let output = Command::new("make")
        .current_dir(&actual_build_dir)
        .output()
        .expect("Failed to build libspot. Make sure you have make installed.");

    if !output.status.success() {
        panic!(
            "Failed to build libspot: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    // Copy built libraries to a known location in OUT_DIR
    let static_lib_versioned = build_dist_dir.join("libspot.a.2.0b3");
    let static_lib_target = Path::new(&out_dir).join("libspot.a");
    let shared_lib_versioned = build_dist_dir.join("libspot.so.2.0b3");
    let shared_lib_target = Path::new(&out_dir).join("libspot.so");

    // Copy the built libraries to OUT_DIR
    if static_lib_versioned.exists() {
        fs::copy(&static_lib_versioned, &static_lib_target)
            .expect("Failed to copy libspot.a to OUT_DIR");
    }
    if shared_lib_versioned.exists() {
        fs::copy(&shared_lib_versioned, &shared_lib_target)
            .expect("Failed to copy libspot.so to OUT_DIR");
    }

    println!("cargo:rerun-if-changed=libspot/");
    println!("cargo:rerun-if-changed=libspot/src/");
    println!("cargo:rerun-if-changed=libspot/Makefile");

    // Tell cargo where to find the library (in OUT_DIR)
    println!("cargo:rustc-link-search=native={}", out_dir);

    // Link against the static library (prefer static over shared for portability)
    println!("cargo:rustc-link-lib=static=spot");

    // Also need to link math library since libspot uses it
    println!("cargo:rustc-link-lib=m");
}
