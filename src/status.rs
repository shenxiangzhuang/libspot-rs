use std::os::raw::c_int;

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