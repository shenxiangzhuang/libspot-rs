#![doc = include_str!("../README.md")]

use std::os::raw::{c_char, c_ulong};

// Module declarations
mod config;
mod detector;
mod error;
mod status;

// Conditional FFI modules
#[cfg(not(target_arch = "wasm32"))]
mod ffi_native;
#[cfg(target_arch = "wasm32")]
mod ffi_wasm;

// Unified FFI interface - this provides a common API regardless of target
#[cfg(not(target_arch = "wasm32"))]
use ffi_native as ffi;
#[cfg(target_arch = "wasm32")]
use ffi_wasm as ffi;

// Re-export public types
pub use config::SpotConfig;
pub use detector::SpotDetector;
pub use error::{SpotError, SpotResult};
pub use status::SpotStatus;

// Re-export commonly used types
pub use std::os::raw::c_double as SpotFloat;

/// Initialize the library
/// 
/// This function must be called before using any other functions in the library.
/// For WASM builds, it sets up the memory allocators.
/// For native builds, it's a no-op but provided for API compatibility.
pub fn init() {
    // Conditionally compiled implementation
    #[cfg(target_arch = "wasm32")]
    {
        ffi::setup_allocators();
    }
    
    // For native builds, no initialization is needed
    #[cfg(not(target_arch = "wasm32"))]
    {
        // No initialization needed for native builds
    }
}

/// Get the version of the underlying libspot library
pub fn version() -> String {
    let mut buffer = vec![0u8; 256];
    
    // Call the FFI function to get the version
    #[cfg(not(target_arch = "wasm32"))]
    unsafe {
        ffi::libspot_version(buffer.as_mut_ptr() as *mut c_char, buffer.len() as c_ulong);
    }
    
    // For WASM, the function is already safe
    #[cfg(target_arch = "wasm32")]
    ffi::libspot_version(buffer.as_mut_ptr() as *mut c_char, buffer.len() as c_ulong);
    
    String::from_utf8_lossy(&buffer)
        .trim_end_matches('\0')
        .to_string()
}

// WASM-specific public exports
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use super::*;
    
    /// WASM-specific initialization function
    #[wasm_bindgen]
    pub fn wasm_init() {
        init();
    }
    
    /// Get the version of the library (WASM-compatible)
    #[wasm_bindgen]
    pub fn wasm_version() -> String {
        version()
    }
}
