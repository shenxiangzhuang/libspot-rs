//! Status codes for SPOT operations

/// Status codes returned by SPOT operations that match the C implementation exactly
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpotStatus {
    /// Data is normal
    Normal = 0,
    /// Data is in the tail (excess)
    Excess = 1,
    /// Data is beyond the anomaly threshold
    Anomaly = 2,
}

impl From<i32> for SpotStatus {
    fn from(code: i32) -> Self {
        match code {
            0 => SpotStatus::Normal,
            1 => SpotStatus::Excess,
            2 => SpotStatus::Anomaly,
            _ => SpotStatus::Normal, // Default fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spot_status_values() {
        // Test that enum values match expected integers
        assert_eq!(SpotStatus::Normal as i32, 0);
        assert_eq!(SpotStatus::Excess as i32, 1);
        assert_eq!(SpotStatus::Anomaly as i32, 2);
    }

    #[test]
    fn test_spot_status_from_i32() {
        // Test conversion from C int values
        assert_eq!(SpotStatus::from(0), SpotStatus::Normal);
        assert_eq!(SpotStatus::from(1), SpotStatus::Excess);
        assert_eq!(SpotStatus::from(2), SpotStatus::Anomaly);

        // Test default fallback for unknown values
        assert_eq!(SpotStatus::from(-1), SpotStatus::Normal);
        assert_eq!(SpotStatus::from(99), SpotStatus::Normal);
    }
}