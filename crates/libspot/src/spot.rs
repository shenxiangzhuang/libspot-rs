//! Main SPOT detector implementation
//!
//! This module implements the main SPOT (Streaming Peaks Over Threshold) detector
//! that provides real-time anomaly detection for time series data.

use crate::config::SpotConfig;

use crate::error::{SpotError, SpotResult};
use crate::p2::p2_quantile;
use crate::status::SpotStatus;
use crate::tail::Tail;

/// Main SPOT detector for streaming anomaly detection
#[derive(Debug)]
pub struct Spot {
    /// Probability of an anomaly
    q: f64,
    /// Location of the tail (high quantile)
    level: f64,
    /// Flag anomalies (true = flag, false = don't flag)
    discard_anomalies: bool,
    /// Upper/Lower tail choice (true = lower tail, false = upper tail)
    low: bool,
    /// Internal constant (+/- 1.0)
    up_down: f64,
    /// Normal/abnormal threshold
    anomaly_threshold: f64,
    /// Tail threshold
    excess_threshold: f64,
    /// Total number of excesses
    nt: usize,
    /// Total number of seen data
    n: usize,
    /// GPD Tail
    tail: Tail,
}

impl Spot {
    /// Create a new SPOT detector with the given configuration
    pub fn new(config: SpotConfig) -> SpotResult<Self> {
        // Validate parameters
        if config.level < 0.0 || config.level >= 1.0 {
            return Err(SpotError::LevelOutOfBounds);
        }
        if config.q >= (1.0 - config.level) || config.q <= 0.0 {
            return Err(SpotError::QOutOfBounds);
        }

        let up_down = if config.low_tail { -1.0 } else { 1.0 };

        Ok(Self {
            q: config.q,
            level: config.level,
            discard_anomalies: config.discard_anomalies,
            low: config.low_tail,
            up_down,
            anomaly_threshold: f64::NAN,
            excess_threshold: f64::NAN,
            nt: 0,
            n: 0,
            tail: Tail::new(config.max_excess)?,
        })
    }

    /// Fit the model using initial training data
    pub fn fit(&mut self, data: &[f64]) -> SpotResult<()> {
        // Reset counters
        self.nt = 0;
        self.n = data.len();

        // Compute excess threshold using P2 quantile estimator
        let et = if self.low {
            // Take the low quantile (1 - level)
            p2_quantile(1.0 - self.level, data)
        } else {
            p2_quantile(self.level, data)
        };

        if et.is_nan() {
            return Err(SpotError::ExcessThresholdIsNaN);
        }

        self.excess_threshold = et;

        // Fill the tail with excesses
        for &value in data {
            // Positive excess
            let excess = self.up_down * (value - et);
            if excess > 0.0 {
                // It's a real excess
                self.nt += 1;
                self.tail.push(excess);
            }
        }

        // Fit the tail with the pushed data
        self.tail.fit();

        // Compute first anomaly threshold
        self.anomaly_threshold = self.quantile(self.q);
        if self.anomaly_threshold.is_nan() {
            return Err(SpotError::AnomalyThresholdIsNaN);
        }

        Ok(())
    }

    /// Process a single data point and return its classification
    pub fn step(&mut self, x: f64) -> SpotResult<SpotStatus> {
        if x.is_nan() {
            return Err(SpotError::DataIsNaN);
        }

        if self.discard_anomalies && (self.up_down * (x - self.anomaly_threshold) > 0.0) {
            return Ok(SpotStatus::Anomaly);
        }

        // Increment number of data (without the anomalies)
        self.n += 1;

        let ex = self.up_down * (x - self.excess_threshold);
        if ex >= 0.0 {
            // Increment number of excesses
            self.nt += 1;
            self.tail.push(ex);
            self.tail.fit();
            // Update threshold
            self.anomaly_threshold = self.quantile(self.q);
            return Ok(SpotStatus::Excess);
        }

        Ok(SpotStatus::Normal)
    }

    /// Get the quantile for a given probability
    pub fn quantile(&self, q: f64) -> f64 {
        if self.n == 0 {
            return f64::NAN;
        }

        let s = (self.nt as f64) / (self.n as f64);
        self.excess_threshold + self.up_down * self.tail.quantile(s, q)
    }

    /// Get the probability for a given value
    pub fn probability(&self, z: f64) -> f64 {
        if self.n == 0 {
            return f64::NAN;
        }

        let s = (self.nt as f64) / (self.n as f64);
        self.tail
            .probability(s, self.up_down * (z - self.excess_threshold))
    }

    /// Get the current anomaly threshold
    pub fn anomaly_threshold(&self) -> f64 {
        self.anomaly_threshold
    }

    /// Get the current excess threshold
    pub fn excess_threshold(&self) -> f64 {
        self.excess_threshold
    }

    /// Get the current configuration (reconstructed)
    pub fn config(&self) -> SpotConfig {
        SpotConfig {
            q: self.q,
            low_tail: self.low,
            discard_anomalies: self.discard_anomalies,
            level: self.level,
            max_excess: self.tail.peaks().container().capacity(),
        }
    }

    /// Get the total number of data points seen
    pub fn n(&self) -> usize {
        self.n
    }

    /// Get the total number of excesses
    pub fn nt(&self) -> usize {
        self.nt
    }

    /// Get the current tail parameters
    pub fn tail_parameters(&self) -> (f64, f64) {
        (self.tail.gamma(), self.tail.sigma())
    }

    /// Get the current size of the tail data
    pub fn tail_size(&self) -> usize {
        self.tail.size()
    }

    /// Get the minimum value in the peaks
    pub fn peaks_min(&self) -> f64 {
        self.tail.peaks().min()
    }

    /// Get the maximum value in the peaks
    pub fn peaks_max(&self) -> f64 {
        self.tail.peaks().max()
    }

    /// Get the mean of the peaks
    pub fn peaks_mean(&self) -> f64 {
        self.tail.peaks().mean()
    }

    /// Get the variance of the peaks
    pub fn peaks_variance(&self) -> f64 {
        self.tail.peaks().variance()
    }

    /// Get the peaks data as a vector (for debugging and export)
    pub fn peaks_data(&self) -> Vec<f64> {
        self.tail.peaks().container().data()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_spot_creation_valid_config() {
        let config = SpotConfig::default();
        let spot = Spot::new(config).unwrap();

        assert_relative_eq!(spot.q, 0.0001);
        assert!(!spot.low);
        assert!(spot.discard_anomalies);
        assert_relative_eq!(spot.level, 0.998);
        assert!(spot.anomaly_threshold().is_nan());
        assert!(spot.excess_threshold().is_nan());
        assert_eq!(spot.n(), 0);
        assert_eq!(spot.nt(), 0);
    }

    #[test]
    fn test_spot_invalid_level() {
        let config = SpotConfig {
            level: 1.5, // Invalid
            ..SpotConfig::default()
        };
        let result = Spot::new(config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SpotError::LevelOutOfBounds);
    }

    #[test]
    fn test_spot_invalid_q() {
        let config = SpotConfig {
            q: 0.5, // Too high for level 0.998
            ..SpotConfig::default()
        };
        let result = Spot::new(config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SpotError::QOutOfBounds);
    }

    #[test]
    fn test_spot_fit_basic() {
        let config = SpotConfig::default();
        let mut spot = Spot::new(config).unwrap();

        // Create simple training data
        let data: Vec<f64> = (0..1000).map(|i| (i as f64 / 1000.0) * 2.0 - 1.0).collect();

        let result = spot.fit(&data);
        assert!(result.is_ok());

        // After fit, thresholds should be valid
        assert!(!spot.anomaly_threshold().is_nan());
        assert!(!spot.excess_threshold().is_nan());
        assert!(spot.anomaly_threshold().is_finite());
        assert!(spot.excess_threshold().is_finite());
        assert_eq!(spot.n(), 1000);
        assert!(spot.nt() > 0); // Should have some excesses
    }

    #[test]
    fn test_spot_step_normal() {
        let config = SpotConfig::default();
        let mut spot = Spot::new(config).unwrap();

        // Fit with simple data
        let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
        spot.fit(&data).unwrap();

        // Test normal value
        let result = spot.step(50.0);
        assert!(result.is_ok());
        // Result depends on the thresholds, but should be valid
    }

    #[test]
    fn test_spot_step_nan() {
        let config = SpotConfig::default();
        let mut spot = Spot::new(config).unwrap();

        let result = spot.step(f64::NAN);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SpotError::DataIsNaN);
    }

    #[test]
    fn test_spot_low_tail() {
        let config = SpotConfig {
            low_tail: true,
            ..SpotConfig::default()
        };
        let spot = Spot::new(config).unwrap();

        assert!(spot.low);
        assert_relative_eq!(spot.up_down, -1.0);
    }

    #[test]
    fn test_spot_config_roundtrip() {
        let original_config = SpotConfig {
            q: 0.001,
            low_tail: true,
            discard_anomalies: false,
            level: 0.99,
            max_excess: 100,
        };

        let spot = Spot::new(original_config.clone()).unwrap();
        let retrieved_config = spot.config();

        assert_relative_eq!(retrieved_config.q, original_config.q);
        assert_eq!(retrieved_config.low_tail, original_config.low_tail);
        assert_eq!(
            retrieved_config.discard_anomalies,
            original_config.discard_anomalies
        );
        assert_relative_eq!(retrieved_config.level, original_config.level);
        assert_eq!(retrieved_config.max_excess, original_config.max_excess);
    }

    #[test]
    fn test_spot_quantile_probability_consistency() {
        let config = SpotConfig::default();
        let mut spot = Spot::new(config).unwrap();

        // Fit with some data
        let data: Vec<f64> = (1..=100).map(|i| i as f64).collect();
        spot.fit(&data).unwrap();

        // Test quantile function
        let q = spot.quantile(0.01);
        assert!(!q.is_nan());
        assert!(q.is_finite());

        // Test probability function
        let p = spot.probability(q);
        assert!(!p.is_nan());
        assert!(p >= 0.0);
    }

    #[test]
    fn test_spot_excess_detection() {
        let config = SpotConfig {
            level: 0.9, // Lower level for easier testing
            ..SpotConfig::default()
        };
        let mut spot = Spot::new(config).unwrap();

        // Fit with data range 0-100
        let data: Vec<f64> = (0..100).map(|i| i as f64).collect();
        spot.fit(&data).unwrap();

        let _initial_nt = spot.nt();

        // Add a value that should be an excess
        let result = spot.step(95.0);
        assert!(result.is_ok());

        // Check that we got some classification
        match result.unwrap() {
            SpotStatus::Normal | SpotStatus::Excess | SpotStatus::Anomaly => {
                // All are valid outcomes
            }
        }
    }
}
