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
    pub max_excess: usize,
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
    fn test_spot_config_new() {
        let config = SpotConfig {
            q: 0.00001,
            low_tail: true,
            discard_anomalies: false,
            level: 0.995,
            max_excess: 500,
        };

        assert_relative_eq!(config.q, 0.00001);
        assert!(config.low_tail);
        assert!(!config.discard_anomalies);
        assert_relative_eq!(config.level, 0.995);
        assert_eq!(config.max_excess, 500);
    }

    #[test]
    fn test_spot_config_debug() {
        let config = SpotConfig::default();
        let debug_str = format!("{config:?}");

        assert!(debug_str.contains("SpotConfig"));
        assert!(debug_str.contains("q: 0.0001"));
        assert!(debug_str.contains("low_tail: false"));
        assert!(debug_str.contains("discard_anomalies: true"));
        assert!(debug_str.contains("level: 0.998"));
        assert!(debug_str.contains("max_excess: 200"));
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

    #[test]
    fn test_spot_config_typical_values() {
        // Test typical configuration values used in anomaly detection
        let high_sensitivity = SpotConfig {
            q: 0.00001,  // Very sensitive
            level: 0.99, // Lower threshold
            ..Default::default()
        };

        let low_sensitivity = SpotConfig {
            q: 0.001,     // Less sensitive
            level: 0.999, // Higher threshold
            ..Default::default()
        };

        assert!(high_sensitivity.q < low_sensitivity.q);
        assert!(high_sensitivity.level < low_sensitivity.level);
    }

    #[test]
    fn test_spot_config_low_tail() {
        let upper_tail = SpotConfig {
            low_tail: false,
            ..Default::default()
        };

        let lower_tail = SpotConfig {
            low_tail: true,
            ..Default::default()
        };

        assert!(!upper_tail.low_tail);
        assert!(lower_tail.low_tail);
    }

    #[test]
    fn test_spot_config_discard_anomalies() {
        let keep_anomalies = SpotConfig {
            discard_anomalies: false,
            ..Default::default()
        };

        let discard_anomalies = SpotConfig {
            discard_anomalies: true,
            ..Default::default()
        };

        assert!(!keep_anomalies.discard_anomalies);
        assert!(discard_anomalies.discard_anomalies);
    }

    #[test]
    fn test_spot_config_max_excess_values() {
        let small_buffer = SpotConfig {
            max_excess: 50,
            ..Default::default()
        };

        let large_buffer = SpotConfig {
            max_excess: 1000,
            ..Default::default()
        };

        assert_eq!(small_buffer.max_excess, 50);
        assert_eq!(large_buffer.max_excess, 1000);
    }
}
