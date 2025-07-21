// This file is a facade that re-exports the appropriate FFI module based on the target architecture.
// It provides a unified interface for both native and WASM builds.

// Re-export the appropriate FFI module based on target architecture
#[cfg(not(target_arch = "wasm32"))]
pub use crate::ffi_native::*;

#[cfg(target_arch = "wasm32")]
pub use crate::ffi_wasm::*;