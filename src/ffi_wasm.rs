use std::os::raw::{c_char, c_double, c_int, c_ulong, c_void};

// Function pointer types for WASM compatibility
pub type MallocFn = unsafe extern "C" fn(size: usize) -> *mut c_void;
pub type FreeFn = unsafe extern "C" fn(ptr: *mut c_void);

// Internal C structures (kept private to this module)
#[repr(C)]
pub struct Ubend {
    pub cursor: c_ulong,
    pub capacity: c_ulong,
    pub last_erased_data: c_double,
    pub filled: c_int,
    pub data: *mut c_double,
}

#[repr(C)]
pub struct Peaks {
    pub e: c_double,
    pub e2: c_double,
    pub min: c_double,
    pub max: c_double,
    pub container: Ubend,
}

#[repr(C)]
pub struct Tail {
    pub gamma: c_double,
    pub sigma: c_double,
    pub peaks: Peaks,
}

#[repr(C)]
pub struct SpotRaw {
    pub q: c_double,
    pub level: c_double,
    pub discard_anomalies: c_int,
    pub low: c_int,
    pub __up_down: c_double,
    pub anomaly_threshold: c_double,
    pub excess_threshold: c_double,
    pub nt: c_ulong,
    pub n: c_ulong,
    pub tail: Tail,
}

// Direct implementation of the FFI functions for WASM
// In a real implementation, these would call into the WASM module

pub fn spot_init(
    _spot: *mut SpotRaw,
    _q: c_double,
    _low: c_int,
    _discard_anomalies: c_int,
    _level: c_double,
    _max_excess: u32,  // Changed from c_ulong to u32 for WASM compatibility
) -> c_int {
    // Simplified implementation for WASM
    // In a real implementation, this would call into the WASM module
    0 // Success
}

pub fn spot_free(_spot: *mut SpotRaw) {
    // Simplified implementation for WASM
}

pub fn spot_fit(_spot: *mut SpotRaw, _data: *const c_double, _size: c_ulong) -> c_int {
    // Simplified implementation for WASM
    0 // Success
}

pub fn spot_step(_spot: *mut SpotRaw, _x: c_double) -> c_int {
    // Simplified implementation for WASM
    0 // Normal (not anomalous)
}

pub fn spot_quantile(_spot: *const SpotRaw, _q: c_double) -> c_double {
    // Simplified implementation for WASM
    0.0
}

pub fn libspot_version(buffer: *mut c_char, size: c_ulong) {
    // Simplified implementation for WASM
    let version = b"libspot 2.0.0-beta.3 (WASM)\0";
    unsafe {
        let len = std::cmp::min(version.len(), size as usize);
        std::ptr::copy_nonoverlapping(version.as_ptr(), buffer as *mut u8, len);
    }
}

pub fn libspot_error(err: c_int, buffer: *mut c_char, size: c_ulong) {
    // Simplified implementation for WASM
    let error_msg = match err {
        -1000 => "Memory allocation failed",
        -1001 => "The level parameter is out of bounds (it must be between 0 and 1, but close to 1)",
        -1002 => "The q parameter must between 0 and 1-level",
        -1003 => "The excess threshold has not been initialized",
        -1004 => "The anomaly threshold has not been initialized",
        -1005 => "The input data is NaN",
        _ => "Unknown error",
    };
    
    // Convert to bytes and add null terminator
    let error_bytes = format!("{}\0", error_msg).into_bytes();
    
    unsafe {
        let len = std::cmp::min(error_bytes.len(), size as usize);
        std::ptr::copy_nonoverlapping(error_bytes.as_ptr(), buffer as *mut u8, len);
    }
}

// Setup allocators for WASM builds
pub fn setup_allocators() {
    // In a real implementation, this would set up the allocators for the WASM module
    // For now, we'll just provide a stub implementation
}