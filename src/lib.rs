#![doc = include_str!("../README.md")]

use std::os::raw::{c_char, c_ulong};

// Module declarations
mod allocator;
mod config;
mod detector;
mod error;
mod ffi;
mod status;

// Re-export public types
pub use config::SpotConfig;
pub use detector::SpotDetector;
pub use error::{SpotError, SpotResult};
pub use status::SpotStatus;

// Re-export commonly used types
pub use std::os::raw::c_double as SpotFloat;

/// Get the version of the underlying libspot library
pub fn version() -> String {
    let mut buffer = vec![0u8; 256];
    unsafe {
        ffi::libspot_version(buffer.as_mut_ptr() as *mut c_char, buffer.len() as c_ulong);
    }
    String::from_utf8_lossy(&buffer)
        .trim_end_matches('\0')
        .to_string()
}
