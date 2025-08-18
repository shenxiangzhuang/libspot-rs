//! Error types for the SPOT algorithm implementation
//!
//! This module defines error types that match the C implementation exactly.

use std::fmt;

/// Result type for SPOT operations
pub type SpotResult<T> = Result<T, SpotError>;

/// Error codes that match the C implementation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpotError {
    /// Memory allocation failed
    MemoryAllocationFailed = 1000,
    /// The level parameter must be between 0 and 1
    LevelOutOfBounds = 1001,
    /// The q parameter must be between 0 and 1-level
    QOutOfBounds = 1002,
    /// The excess threshold has not been initialized
    ExcessThresholdIsNaN = 1003,
    /// The anomaly threshold has not been initialized
    AnomalyThresholdIsNaN = 1004,
    /// The input data is NaN
    DataIsNaN = 1005,
}

impl SpotError {
    /// Convert from C error code
    pub fn from_code(code: i32) -> Self {
        match code.abs() {
            1000 => SpotError::MemoryAllocationFailed,
            1001 => SpotError::LevelOutOfBounds,
            1002 => SpotError::QOutOfBounds,
            1003 => SpotError::ExcessThresholdIsNaN,
            1004 => SpotError::AnomalyThresholdIsNaN,
            1005 => SpotError::DataIsNaN,
            _ => SpotError::MemoryAllocationFailed, // Default fallback
        }
    }

    /// Get error message
    pub fn message(&self) -> &'static str {
        match self {
            SpotError::MemoryAllocationFailed => "Memory allocation failed",
            SpotError::LevelOutOfBounds => {
                "The level parameter is out of bounds (it must be between 0 and 1, but close to 1)"
            }
            SpotError::QOutOfBounds => "The q parameter must between 0 and 1-level",
            SpotError::ExcessThresholdIsNaN => "The excess threshold has not been initialized",
            SpotError::AnomalyThresholdIsNaN => "The anomaly threshold has not been initialized",
            SpotError::DataIsNaN => "The input data is NaN",
        }
    }

    /// Get error code
    pub fn code(&self) -> i32 {
        *self as i32
    }
}

impl fmt::Display for SpotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for SpotError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes_match_c() {
        assert_eq!(SpotError::MemoryAllocationFailed.code(), 1000);
        assert_eq!(SpotError::LevelOutOfBounds.code(), 1001);
        assert_eq!(SpotError::QOutOfBounds.code(), 1002);
        assert_eq!(SpotError::ExcessThresholdIsNaN.code(), 1003);
        assert_eq!(SpotError::AnomalyThresholdIsNaN.code(), 1004);
        assert_eq!(SpotError::DataIsNaN.code(), 1005);
    }

    #[test]
    fn test_from_code() {
        assert_eq!(
            SpotError::from_code(-1000),
            SpotError::MemoryAllocationFailed
        );
        assert_eq!(SpotError::from_code(-1001), SpotError::LevelOutOfBounds);
        assert_eq!(SpotError::from_code(-1002), SpotError::QOutOfBounds);
        assert_eq!(SpotError::from_code(-1003), SpotError::ExcessThresholdIsNaN);
        assert_eq!(
            SpotError::from_code(-1004),
            SpotError::AnomalyThresholdIsNaN
        );
        assert_eq!(SpotError::from_code(-1005), SpotError::DataIsNaN);
    }

    #[test]
    fn test_error_messages() {
        assert_eq!(
            SpotError::MemoryAllocationFailed.message(),
            "Memory allocation failed"
        );
        assert_eq!(
            SpotError::LevelOutOfBounds.message(),
            "The level parameter is out of bounds (it must be between 0 and 1, but close to 1)"
        );
    }

    #[test]
    fn test_error_display() {
        let error = SpotError::DataIsNaN;
        assert_eq!(format!("{}", error), "The input data is NaN");
    }
}
