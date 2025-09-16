use std::env;
use std::path::Path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let libspot_dir = Path::new(&manifest_dir).join("libspot");

    // Extract version from Makefile
    let makefile_path = libspot_dir.join("Makefile");
    let makefile_content =
        std::fs::read_to_string(&makefile_path).expect("Failed to read Makefile");

    let version = makefile_content
        .lines()
        .find(|line| line.trim().starts_with("VERSION") && line.contains("="))
        .and_then(|line| line.split('=').nth(1))
        .map(|v| v.trim())
        .expect("Failed to extract VERSION from Makefile");

    // Build libspot from C sources
    let mut build = cc::Build::new();

    // Add all C source files
    let src_dir = libspot_dir.join("src");
    for entry in std::fs::read_dir(&src_dir).expect("Failed to read src directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "c") {
            build.file(&path);
        }
    }

    // Set compiler flags matching the Makefile
    let version_define = format!("\"{version}\"");
    build
        .include(libspot_dir.join("include"))
        .opt_level(3)
        .flag("-std=c99")
        .define("VERSION", version_define.as_str())
        .warnings(true)
        .extra_warnings(true);

    // Compile as static library
    build.compile("spot");

    // Link against the math library (required by libspot)
    println!("cargo:rustc-link-lib=m");

    // Rerun build script if libspot source files change
    println!("cargo:rerun-if-changed=libspot/src/");
    println!("cargo:rerun-if-changed=libspot/include/");
    println!("cargo:rerun-if-changed=libspot/Makefile");
}
