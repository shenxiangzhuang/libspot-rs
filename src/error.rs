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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spot_error_from_code() {
        // Test all known error codes
        assert_eq!(SpotError::from_code(-1000), SpotError::MemoryAllocationFailed);
        assert_eq!(SpotError::from_code(-1001), SpotError::LevelOutOfBounds);
        assert_eq!(SpotError::from_code(-1002), SpotError::QOutOfBounds);
        assert_eq!(SpotError::from_code(-1003), SpotError::ExcessThresholdIsNaN);
        assert_eq!(SpotError::from_code(-1004), SpotError::AnomalyThresholdIsNaN);
        assert_eq!(SpotError::from_code(-1005), SpotError::DataIsNaN);
        
        // Test unknown error code
        assert_eq!(SpotError::from_code(-9999), SpotError::Unknown(-9999));
    }

    #[test]
    fn test_spot_error_code() {
        // Test that error codes match expected values
        assert_eq!(SpotError::MemoryAllocationFailed.code(), -1000);
        assert_eq!(SpotError::LevelOutOfBounds.code(), -1001);
        assert_eq!(SpotError::QOutOfBounds.code(), -1002);
        assert_eq!(SpotError::ExcessThresholdIsNaN.code(), -1003);
        assert_eq!(SpotError::AnomalyThresholdIsNaN.code(), -1004);
        assert_eq!(SpotError::DataIsNaN.code(), -1005);
        assert_eq!(SpotError::NotInitialized.code(), -1);
        assert_eq!(SpotError::Unknown(-9999).code(), -9999);
    }

    #[test]
    fn test_spot_error_display() {
        // Test that display messages match expected strings
        assert_eq!(
            SpotError::MemoryAllocationFailed.to_string(),
            "Memory allocation failed"
        );
        assert_eq!(
            SpotError::LevelOutOfBounds.to_string(),
            "The level parameter is out of bounds (it must be between 0 and 1, but close to 1)"
        );
        assert_eq!(
            SpotError::QOutOfBounds.to_string(),
            "The q parameter must between 0 and 1-level"
        );
        assert_eq!(
            SpotError::ExcessThresholdIsNaN.to_string(),
            "The excess threshold has not been initialized"
        );
        assert_eq!(
            SpotError::AnomalyThresholdIsNaN.to_string(),
            "The anomaly threshold has not been initialized"
        );
        assert_eq!(
            SpotError::DataIsNaN.to_string(),
            "The input data is NaN"
        );
        assert_eq!(
            SpotError::NotInitialized.to_string(),
            "Detector not initialized"
        );
        assert_eq!(
            SpotError::Unknown(-9999).to_string(),
            "Unknown error (code: -9999)"
        );
    }

    #[test]
    fn test_spot_error_message() {
        // Test that NotInitialized returns the expected message
        assert_eq!(
            SpotError::NotInitialized.message(),
            "Detector not initialized"
        );
        
        // Note: We can't easily test the C library message function here
        // without linking to the C library, but we can test the structure
    }

    #[test]
    fn test_spot_error_clone_and_eq() {
        let error1 = SpotError::LevelOutOfBounds;
        let error2 = error1.clone();
        assert_eq!(error1, error2);
        
        let error3 = SpotError::QOutOfBounds;
        assert_ne!(error1, error3);
    }

    #[test]
    fn test_spot_error_debug() {
        let error = SpotError::LevelOutOfBounds;
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("LevelOutOfBounds"));
    }

    #[test]
    fn test_spot_result_type() {
        let result_ok: SpotResult<i32> = Ok(42);
        let result_err: SpotResult<i32> = Err(SpotError::LevelOutOfBounds);
        
        assert!(result_ok.is_ok());
        assert!(result_err.is_err());
        assert_eq!(result_err.unwrap_err(), SpotError::LevelOutOfBounds);
    }

    #[test]
    fn test_spot_error_is_error() {
        let error = SpotError::LevelOutOfBounds;
        // Test that it implements the Error trait
        let _: &dyn std::error::Error = &error;
    }
} 