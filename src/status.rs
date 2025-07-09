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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spot_status_values() {
        // Test that enum values match expected integers
        assert_eq!(SpotStatus::Normal as c_int, 0);
        assert_eq!(SpotStatus::Excess as c_int, 1);
        assert_eq!(SpotStatus::Anomaly as c_int, 2);
    }

    #[test]
    fn test_spot_status_from_c_int() {
        // Test conversion from C int values
        assert_eq!(SpotStatus::from(0), SpotStatus::Normal);
        assert_eq!(SpotStatus::from(1), SpotStatus::Excess);
        assert_eq!(SpotStatus::from(2), SpotStatus::Anomaly);
        
        // Test default fallback for unknown values
        assert_eq!(SpotStatus::from(-1), SpotStatus::Normal);
        assert_eq!(SpotStatus::from(99), SpotStatus::Normal);
    }

    #[test]
    fn test_spot_status_debug() {
        // Test debug representation
        assert_eq!(format!("{:?}", SpotStatus::Normal), "Normal");
        assert_eq!(format!("{:?}", SpotStatus::Excess), "Excess");
        assert_eq!(format!("{:?}", SpotStatus::Anomaly), "Anomaly");
    }

    #[test]
    fn test_spot_status_equality() {
        // Test equality and inequality
        assert_eq!(SpotStatus::Normal, SpotStatus::Normal);
        assert_eq!(SpotStatus::Excess, SpotStatus::Excess);
        assert_eq!(SpotStatus::Anomaly, SpotStatus::Anomaly);
        
        assert_ne!(SpotStatus::Normal, SpotStatus::Excess);
        assert_ne!(SpotStatus::Excess, SpotStatus::Anomaly);
        assert_ne!(SpotStatus::Normal, SpotStatus::Anomaly);
    }

    #[test]
    fn test_spot_status_copy_clone() {
        // Test that SpotStatus is Copy and Clone
        let status1 = SpotStatus::Excess;
        let status2 = status1; // Copy
        let status3 = status1.clone(); // Clone
        
        assert_eq!(status1, status2);
        assert_eq!(status1, status3);
    }

    #[test]
    fn test_spot_status_match() {
        // Test pattern matching
        let status = SpotStatus::Excess;
        
        let result = match status {
            SpotStatus::Normal => "normal",
            SpotStatus::Excess => "excess",
            SpotStatus::Anomaly => "anomaly",
        };
        
        assert_eq!(result, "excess");
    }
} 