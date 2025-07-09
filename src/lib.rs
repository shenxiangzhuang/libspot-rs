//! # libspot
//!
//! A Rust wrapper for the libspot anomaly detection library.
//!
//! This crate provides safe Rust bindings for the libspot library, which implements
//! the SPOT (Streaming Peaks Over Threshold) algorithm for anomaly detection.

use std::fmt;
use std::mem::MaybeUninit;
use std::os::raw::{c_char, c_double, c_int, c_ulong, c_void};

// Re-export commonly used types
pub use std::os::raw::c_double as SpotFloat;

/// Result type for SPOT operations
pub type SpotResult<T> = Result<T, SpotError>;

/// Status codes returned by SPOT operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpotStatus {
    /// Data is normal
    Normal = 0,
    /// Data is in the tail (excess)
    Excess = 1,
    /// Data is beyond the anomaly threshold
    Anomaly = 2,
}

impl From<c_int> for SpotStatus {
    fn from(code: c_int) -> Self {
        match code {
            0 => SpotStatus::Normal,
            1 => SpotStatus::Excess,
            2 => SpotStatus::Anomaly,
            _ => SpotStatus::Normal, // Default fallback
        }
    }
}

/// Errors that can occur during SPOT operations
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpotError {
    /// Memory allocation failed
    MemoryAllocationFailed,
    /// The level parameter is out of bounds (must be between 0 and 1)
    LevelOutOfBounds,
    /// The q parameter is out of bounds (must be between 0 and 1-level)
    QOutOfBounds,
    /// The excess threshold has not been initialized
    ExcessThresholdIsNaN,
    /// The anomaly threshold has not been initialized
    AnomalyThresholdIsNaN,
    /// The input data is NaN
    DataIsNaN,
    /// Detector not initialized
    NotInitialized,
    /// Unknown error with code
    Unknown(c_int),
}

impl SpotError {
    /// Create a SpotError from a C error code
    pub fn from_code(code: c_int) -> Self {
        match code {
            -1000 => SpotError::MemoryAllocationFailed,
            -1001 => SpotError::LevelOutOfBounds,
            -1002 => SpotError::QOutOfBounds,
            -1003 => SpotError::ExcessThresholdIsNaN,
            -1004 => SpotError::AnomalyThresholdIsNaN,
            -1005 => SpotError::DataIsNaN,
            _ => SpotError::Unknown(code),
        }
    }

    /// Get the error code
    pub fn code(&self) -> c_int {
        match self {
            SpotError::MemoryAllocationFailed => -1000,
            SpotError::LevelOutOfBounds => -1001,
            SpotError::QOutOfBounds => -1002,
            SpotError::ExcessThresholdIsNaN => -1003,
            SpotError::AnomalyThresholdIsNaN => -1004,
            SpotError::DataIsNaN => -1005,
            SpotError::NotInitialized => -1,
            SpotError::Unknown(code) => *code,
        }
    }

    /// Get the error message from the C library
    pub fn message(&self) -> String {
        if let SpotError::NotInitialized = self {
            return "Detector not initialized".to_string();
        }

        unsafe {
            let mut buffer = vec![0u8; 256];
            libspot_error(
                self.code(),
                buffer.as_mut_ptr() as *mut c_char,
                buffer.len() as c_ulong,
            );
            String::from_utf8_lossy(&buffer)
                .trim_end_matches('\0')
                .to_string()
        }
    }
}

impl fmt::Display for SpotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpotError::MemoryAllocationFailed => write!(f, "Memory allocation failed"),
            SpotError::LevelOutOfBounds => write!(
                f,
                "The level parameter is out of bounds (it must be between 0 and 1, but close to 1)"
            ),
            SpotError::QOutOfBounds => write!(f, "The q parameter must between 0 and 1-level"),
            SpotError::ExcessThresholdIsNaN => {
                write!(f, "The excess threshold has not been initialized")
            }
            SpotError::AnomalyThresholdIsNaN => {
                write!(f, "The anomaly threshold has not been initialized")
            }
            SpotError::DataIsNaN => write!(f, "The input data is NaN"),
            SpotError::NotInitialized => write!(f, "Detector not initialized"),
            SpotError::Unknown(code) => write!(f, "Unknown error (code: {})", code),
        }
    }
}

impl std::error::Error for SpotError {}

// Function pointer types
type MallocFn = unsafe extern "C" fn(size: usize) -> *mut c_void;
type FreeFn = unsafe extern "C" fn(ptr: *mut c_void);

// Internal C structures (kept private)
#[repr(C)]
struct Ubend {
    cursor: c_ulong,
    capacity: c_ulong,
    last_erased_data: c_double,
    filled: c_int,
    data: *mut c_double,
}

#[repr(C)]
struct Peaks {
    e: c_double,
    e2: c_double,
    min: c_double,
    max: c_double,
    container: Ubend,
}

#[repr(C)]
struct Tail {
    gamma: c_double,
    sigma: c_double,
    peaks: Peaks,
}

#[repr(C)]
struct SpotRaw {
    q: c_double,
    level: c_double,
    discard_anomalies: c_int,
    low: c_int,
    __up_down: c_double,
    anomaly_threshold: c_double,
    excess_threshold: c_double,
    nt: c_ulong,
    n: c_ulong,
    tail: Tail,
}

// FFI declarations
#[link(name = "spot")]
extern "C" {
    fn spot_init(
        spot: *mut SpotRaw,
        q: c_double,
        low: c_int,
        discard_anomalies: c_int,
        level: c_double,
        max_excess: c_ulong,
    ) -> c_int;
    fn spot_free(spot: *mut SpotRaw);
    fn spot_fit(spot: *mut SpotRaw, data: *const c_double, size: c_ulong) -> c_int;
    fn spot_step(spot: *mut SpotRaw, x: c_double) -> c_int;
    fn spot_quantile(spot: *const SpotRaw, q: c_double) -> c_double;
    fn libspot_version(buffer: *mut c_char, size: c_ulong);
    fn libspot_error(err: c_int, buffer: *mut c_char, size: c_ulong);
    fn set_allocators(m: MallocFn, f: FreeFn);
}

/// Configuration for initializing a SPOT detector
#[derive(Debug, Clone)]
pub struct SpotConfig {
    /// Decision probability (SPOT will flag extreme events with probability lower than this)
    pub q: f64,
    /// Lower tail mode (false for upper tail, true for lower tail)
    pub low_tail: bool,
    /// Do not include anomalies in the model
    pub discard_anomalies: bool,
    /// Excess level (high quantile that delimits the tail)
    pub level: f64,
    /// Maximum number of data points kept to analyze the tail
    pub max_excess: u64,
}

impl Default for SpotConfig {
    fn default() -> Self {
        SpotConfig {
            q: 0.0001,
            low_tail: false,
            discard_anomalies: true,
            level: 0.998,
            max_excess: 200,
        }
    }
}

/// A SPOT detector for streaming anomaly detection
pub struct SpotDetector {
    raw: MaybeUninit<SpotRaw>,
    initialized: bool,
}

impl SpotDetector {
    /// Create a new SPOT detector with the given configuration
    pub fn new(config: SpotConfig) -> SpotResult<Self> {
        // Initialize allocators
        unsafe {
            set_allocators(libc::malloc, libc::free);
        }

        let mut detector = SpotDetector {
            raw: MaybeUninit::uninit(),
            initialized: false,
        };

        unsafe {
            let status = spot_init(
                detector.raw.as_mut_ptr(),
                config.q,
                if config.low_tail { 1 } else { 0 },
                if config.discard_anomalies { 1 } else { 0 },
                config.level,
                config.max_excess,
            );

            if status < 0 {
                return Err(SpotError::from_code(status));
            }
        }

        detector.initialized = true;
        Ok(detector)
    }

    /// Fit the model using initial training data
    pub fn fit(&mut self, data: &[f64]) -> SpotResult<()> {
        if !self.initialized {
            return Err(SpotError::NotInitialized);
        }

        unsafe {
            let status = spot_fit(self.raw.as_mut_ptr(), data.as_ptr(), data.len() as c_ulong);

            if status < 0 {
                return Err(SpotError::from_code(status));
            }
        }

        Ok(())
    }

    /// Process a single data point and return its classification
    pub fn step(&mut self, value: f64) -> SpotResult<SpotStatus> {
        if !self.initialized {
            return Err(SpotError::NotInitialized);
        }

        unsafe {
            let status = spot_step(self.raw.as_mut_ptr(), value);
            if status < 0 {
                return Err(SpotError::from_code(status));
            }
            Ok(SpotStatus::from(status))
        }
    }

    /// Get the quantile for a given probability
    pub fn quantile(&self, q: f64) -> f64 {
        if !self.initialized {
            return f64::NAN;
        }

        unsafe { spot_quantile(self.raw.as_ptr(), q) }
    }

    /// Get the current anomaly threshold
    pub fn anomaly_threshold(&self) -> f64 {
        if !self.initialized {
            return f64::NAN;
        }

        unsafe {
            let spot_ref = &*self.raw.as_ptr();
            spot_ref.anomaly_threshold
        }
    }

    /// Get the current excess threshold
    pub fn excess_threshold(&self) -> f64 {
        if !self.initialized {
            return f64::NAN;
        }

        unsafe {
            let spot_ref = &*self.raw.as_ptr();
            spot_ref.excess_threshold
        }
    }

    /// Get the current configuration parameters
    pub fn config(&self) -> Option<SpotConfig> {
        if !self.initialized {
            return None;
        }

        unsafe {
            let spot_ref = &*self.raw.as_ptr();
            Some(SpotConfig {
                q: spot_ref.q,
                low_tail: spot_ref.low != 0,
                discard_anomalies: spot_ref.discard_anomalies != 0,
                level: spot_ref.level,
                max_excess: spot_ref.tail.peaks.container.capacity,
            })
        }
    }
}

impl Drop for SpotDetector {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                spot_free(self.raw.as_mut_ptr());
            }
        }
    }
}

// Safety: SpotDetector can be safely sent between threads as long as it's not used concurrently
unsafe impl Send for SpotDetector {}

/// Get the version of the underlying libspot library
pub fn version() -> String {
    let mut buffer = vec![0u8; 256];
    unsafe {
        libspot_version(buffer.as_mut_ptr() as *mut c_char, buffer.len() as c_ulong);
    }
    String::from_utf8_lossy(&buffer)
        .trim_end_matches('\0')
        .to_string()
}
