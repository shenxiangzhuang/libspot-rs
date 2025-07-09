use std::fmt;
use std::os::raw::{c_char, c_int, c_ulong};

/// Result type for SPOT operations
pub type SpotResult<T> = Result<T, SpotError>;

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

// FFI declaration needed for the message() method
extern "C" {
    fn libspot_error(err: c_int, buffer: *mut c_char, size: c_ulong);
} 