use std::os::raw::{c_char, c_double, c_int, c_ulong, c_void};

// Function pointer types
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

// FFI declarations
#[link(name = "spot", kind = "static")]
extern "C" {
    pub fn spot_init(
        spot: *mut SpotRaw,
        q: c_double,
        low: c_int,
        discard_anomalies: c_int,
        level: c_double,
        max_excess: c_ulong,
    ) -> c_int;
    pub fn spot_free(spot: *mut SpotRaw);
    pub fn spot_fit(spot: *mut SpotRaw, data: *const c_double, size: c_ulong) -> c_int;
    pub fn spot_step(spot: *mut SpotRaw, x: c_double) -> c_int;
    pub fn spot_quantile(spot: *const SpotRaw, q: c_double) -> c_double;
    pub fn libspot_version(buffer: *mut c_char, size: c_ulong);
    // pub fn libspot_error(err: c_int, buffer: *mut c_char, size: c_ulong);
    pub fn set_allocators(m: MallocFn, f: FreeFn);
    
    // GPD estimators
    pub fn grimshaw_estimator(peaks: *const Peaks, gamma: *mut c_double, sigma: *mut c_double) -> c_double;
}
