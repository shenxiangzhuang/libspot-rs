//! Configuration types for SPOT detector

/// Configuration parameters for SPOT detector
#[derive(Debug, Clone, PartialEq)]
pub struct SpotConfig {
    /// Anomaly probability threshold (must be between 0 and 1-level)
    pub q: f64,
    /// Whether to observe lower tail (false = upper tail, true = lower tail)
    pub low_tail: bool,
    /// Whether to discard anomalies from model updates
    pub discard_anomalies: bool,
    /// Excess level - high quantile that delimits the tail (must be between 0 and 1)
    pub level: f64,
    /// Maximum number of excess data points to keep
    pub max_excess: usize,
}

impl Default for SpotConfig {
    /// Default configuration that matches the C implementation
    fn default() -> Self {
        Self {
            q: 0.0001,
            low_tail: false,
            discard_anomalies: true,
            level: 0.998,
            max_excess: 200,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_spot_config_default() {
        let config = SpotConfig::default();

        assert_relative_eq!(config.q, 0.0001);
        assert!(!config.low_tail);
        assert!(config.discard_anomalies);
        assert_relative_eq!(config.level, 0.998);
        assert_eq!(config.max_excess, 200);
    }

    #[test]
    fn test_spot_config_clone() {
        let config1 = SpotConfig::default();
        let config2 = config1.clone();

        assert_relative_eq!(config1.q, config2.q);
        assert_eq!(config1.low_tail, config2.low_tail);
        assert_eq!(config1.discard_anomalies, config2.discard_anomalies);
        assert_relative_eq!(config1.level, config2.level);
        assert_eq!(config1.max_excess, config2.max_excess);
    }
}
