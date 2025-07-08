use std::process::Command;
use std::env;
use std::path::Path;
use std::fs;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    
    // Path to the libspot directory
    let libspot_dir = Path::new(&manifest_dir).join("libspot");
    let dist_dir = libspot_dir.join("dist");
    
    // Build libspot using make
    let output = Command::new("make")
        .current_dir(&libspot_dir)
        .output()
        .expect("Failed to build libspot. Make sure you have make installed.");
    
    if !output.status.success() {
        panic!("Failed to build libspot: {}", String::from_utf8_lossy(&output.stderr));
    }
    
    // Create symlinks without version numbers so Rust can find them
    let static_lib_versioned = dist_dir.join("libspot.a.2.0b3");
    let static_lib = dist_dir.join("libspot.a");
    let shared_lib_versioned = dist_dir.join("libspot.so.2.0b3");
    let shared_lib = dist_dir.join("libspot.so");
    
    // Remove old symlinks if they exist
    let _ = fs::remove_file(&static_lib);
    let _ = fs::remove_file(&shared_lib);
    
    // Create new symlinks
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(&static_lib_versioned, &static_lib)
            .expect("Failed to create symlink for libspot.a");
        std::os::unix::fs::symlink(&shared_lib_versioned, &shared_lib)
            .expect("Failed to create symlink for libspot.so");
    }
    
    println!("cargo:rerun-if-changed=libspot/");
    println!("cargo:rerun-if-changed=libspot/src/");
    println!("cargo:rerun-if-changed=libspot/Makefile");
    
    // Tell cargo where to find the library
    println!("cargo:rustc-link-search=native={}", dist_dir.display());
    
    // Link against the static library (prefer static over shared for portability)
    println!("cargo:rustc-link-lib=static=spot");
    
    // Also need to link math library since libspot uses it
    println!("cargo:rustc-link-lib=m");
} 