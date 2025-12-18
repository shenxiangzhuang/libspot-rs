#![doc = include_str!("../README.md")]

use std::os::raw::{c_char, c_ulong};

// Module declarations
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

// Re-export function pointer types for advanced users
pub use ffi::{FreeFn, FrexpFn, LdexpFn, MallocFn, Math2Fn, MathFn};

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

/// Set custom math functions (log, exp, pow) for the library.
///
/// This is useful when you have access to optimized versions of these functions
/// (e.g., from the standard library or specialized math libraries).
///
/// Pass `None` for any function you don't want to change.
///
/// # Safety
/// The provided function pointers must be valid and thread-safe.
///
/// # Example
/// ```ignore
/// // Use standard library math functions
/// unsafe extern "C" fn log_fn(x: f64) -> f64 { x.ln() }
/// unsafe extern "C" fn exp_fn(x: f64) -> f64 { x.exp() }
/// unsafe extern "C" fn pow_fn(x: f64, y: f64) -> f64 { x.powf(y) }
///
/// unsafe {
///     libspot::set_math_functions(Some(log_fn), Some(exp_fn), Some(pow_fn));
/// }
/// ```
pub unsafe fn set_math_functions(log: Option<MathFn>, exp: Option<MathFn>, pow: Option<Math2Fn>) {
    ffi::set_math_functions(log, exp, pow);
}

/// Set custom ldexp/frexp functions for float decomposition.
///
/// These are low-level functions for floating point manipulation.
/// Most users should use `set_math_functions` instead.
///
/// Pass `None` for any function you don't want to change.
///
/// # Safety
/// The provided function pointers must be valid and thread-safe.
pub unsafe fn set_float_utils(ldexp: Option<LdexpFn>, frexp: Option<FrexpFn>) {
    ffi::set_float_utils(ldexp, frexp);
}
