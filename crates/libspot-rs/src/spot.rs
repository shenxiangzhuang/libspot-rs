//! Main SPOT detector implementation
//!
//! This module implements the main SPOT (Streaming Peaks Over Threshold) detector
//! that provides real-time anomaly detection for time series data.
//!
//! # Serialization
//!
//! When the `serde` feature is enabled, the [`SpotDetector`] can be serialized and
//! deserialized. This is particularly useful for:
//!
//! - **Model persistence**: Save a trained model to disk and load it later
//! - **Model deployment**: Export models for use in production systems
//! - **Model sharing**: Share trained models between different applications
//! - **Checkpointing**: Save model state during long-running processes
//!
//! ## Example
//!
//! ```ignore
//! use libspot_rs::{SpotConfig, SpotDetector};
//! use serde_json;
//!
//! // Train a model
//! let config = SpotConfig::default();
//! let mut spot = SpotDetector::new(config).unwrap();
//! let training_data: Vec<f64> = (0..1000).map(|i| i as f64 / 100.0).collect();
//! spot.fit(&training_data).unwrap();
//!
//! // Serialize the trained model
//! let json = serde_json::to_string(&spot).unwrap();
//!
//! // Later, deserialize and continue using
//! let loaded: SpotDetector = serde_json::from_str(&json).unwrap();
//! let status = loaded.step(50.0);
//! ```

use crate::config::{SpotConfig, SpotEstimator, SpotExcessUpdate, SpotInitialThreshold};

use crate::error::{SpotError, SpotResult};
use crate::p2::p2_quantile;
use crate::status::SpotStatus;
use crate::tail::Tail;

/// Main SPOT detector for streaming anomaly detection
///
/// The `SpotDetector` implements the SPOT (Streaming Peaks Over Threshold) algorithm
/// for real-time anomaly detection in streaming time series data.
///
/// # Serialization
///
/// When the `serde` feature is enabled, the detector can be serialized and deserialized,
/// allowing you to save trained models and restore them later without re-training.
///
/// # Example
///
/// ```
/// use libspot_rs::{SpotConfig, SpotDetector, SpotStatus};
///
/// let config = SpotConfig::default();
/// let mut spot = SpotDetector::new(config).unwrap();
///
/// // Fit with training data
/// let data: Vec<f64> = (0..1000).map(|i| (i as f64) / 100.0).collect();
/// spot.fit(&data).unwrap();
///
/// // Process new data points
/// match spot.step(15.0).unwrap() {
///     SpotStatus::Normal => println!("Normal"),
///     SpotStatus::Excess => println!("Excess"),
///     SpotStatus::Anomaly => println!("Anomaly detected!"),
/// }
/// ```
#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SpotDetector {
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
    #[cfg_attr(feature = "serde", serde(with = "crate::ser::nan_safe_f64"))]
    anomaly_threshold: f64,
    /// Tail threshold
    #[cfg_attr(feature = "serde", serde(with = "crate::ser::nan_safe_f64"))]
    excess_threshold: f64,
    /// Total number of excesses
    nt: usize,
    /// Total number of seen data
    n: usize,
    /// GPD parameter estimator policy
    #[cfg_attr(feature = "serde", serde(default))]
    estimator: SpotEstimator,
    /// Initial excess threshold selection strategy
    #[cfg_attr(feature = "serde", serde(default))]
    initial_threshold: SpotInitialThreshold,
    /// Streaming excess update condition
    #[cfg_attr(feature = "serde", serde(default))]
    excess_update: SpotExcessUpdate,
    /// GPD Tail
    tail: Tail,
}

impl SpotDetector {
    /// Create a new SPOT detector with the given configuration
    pub fn new(config: SpotConfig) -> SpotResult<Self> {
        Self::new_with_estimator(config, SpotEstimator::default())
    }

    /// Create a new SPOT detector with the given configuration and estimator.
    pub fn new_with_estimator(config: SpotConfig, estimator: SpotEstimator) -> SpotResult<Self> {
        Self::new_with_options(config, estimator, SpotInitialThreshold::default())
    }

    /// Create a new SPOT detector with explicit estimator and initial threshold options.
    pub fn new_with_options(
        config: SpotConfig,
        estimator: SpotEstimator,
        initial_threshold: SpotInitialThreshold,
    ) -> SpotResult<Self> {
        Self::new_with_full_options(
            config,
            estimator,
            initial_threshold,
            SpotExcessUpdate::default(),
        )
    }

    /// Create a new SPOT detector with all algorithm options explicit.
    pub fn new_with_full_options(
        config: SpotConfig,
        estimator: SpotEstimator,
        initial_threshold: SpotInitialThreshold,
        excess_update: SpotExcessUpdate,
    ) -> SpotResult<Self> {
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
            estimator,
            initial_threshold,
            excess_update,
            tail: Tail::new(config.max_excess)?,
        })
    }

    /// Fit the model using initial training data
    pub fn fit(&mut self, data: &[f64]) -> SpotResult<()> {
        // Reset learned state so repeated fits behave like fitting a fresh detector.
        self.nt = 0;
        self.n = data.len();
        self.tail.reset();

        // Compute the initial excess threshold.
        let et = if self.low {
            // Take the low quantile (1 - level)
            self.initial_quantile(1.0 - self.level, data)
        } else {
            self.initial_quantile(self.level, data)
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
        self.tail.fit_with(self.estimator);

        // Compute first anomaly threshold
        self.anomaly_threshold = self.quantile(self.q);
        if self.anomaly_threshold.is_nan() {
            return Err(SpotError::AnomalyThresholdIsNaN);
        }

        Ok(())
    }

    /// Process a single data point and return its classification
    pub fn step(&mut self, value: f64) -> SpotResult<SpotStatus> {
        if value.is_nan() {
            return Err(SpotError::DataIsNaN);
        }

        if self.discard_anomalies && (self.up_down * (value - self.anomaly_threshold) > 0.0) {
            return Ok(SpotStatus::Anomaly);
        }

        // Increment number of data (without the anomalies)
        self.n += 1;

        let ex = self.up_down * (value - self.excess_threshold);
        if self.should_update_excess(ex) {
            // Increment number of excesses
            self.nt += 1;
            self.tail.push(ex);
            self.tail.fit_with(self.estimator);
            // Update threshold
            self.anomaly_threshold = self.quantile(self.q);
            return Ok(SpotStatus::Excess);
        }

        Ok(SpotStatus::Normal)
    }

    /// Record a normal observation without updating the tail.
    ///
    /// This is useful when the caller has already decided to suppress a
    /// missing or invalid sample but still wants the stream length to advance.
    /// The regular [`step`](Self::step) API remains strict about `NaN` inputs.
    pub fn observe_normal(&mut self) {
        self.n += 1;
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

    /// Return a bounded anomaly score derived from the fitted GPD tail.
    ///
    /// The score is the conditional percentile of `value` inside the selected
    /// tail. It is defined from the Generalized Pareto survival function as:
    ///
    /// $$
    /// y(x) = a(x-t), \qquad
    /// a = \begin{cases} 1 & \text{upper tail}, \\\\ -1 & \text{lower tail}. \end{cases}
    /// $$
    ///
    /// $$
    /// A(x) = \begin{cases}
    /// 0, & y \le 0, \\\\
    /// 1-\left(1+\dfrac{\gamma y}{\sigma}\right)^{-1/\gamma},
    ///     & y>0,\ \gamma\ne0, \\\\
    /// 1-\exp\left(-\dfrac{y}{\sigma}\right), & y>0,\ \gamma=0.
    /// \end{cases}
    /// $$
    ///
    /// `up_down` is `1` for the upper tail and `-1` for the lower tail, so
    /// the same equation applies to both modes.
    ///
    /// # Interpretation
    ///
    /// - `0` means that `value` has not entered the modeled tail.
    /// - Values closer to `1` are more extreme within that tail.
    /// - At the current SPOT anomaly threshold $z_q$, the score is
    ///   $A(z_q)=1-q/s$, where $s=N_t/n$ is the observed tail fraction.
    ///
    /// This is a tail percentile, not the posterior probability that the
    /// observation is anomalous. Classification and model updates remain the
    /// responsibility of [`step`](Self::step); this method never mutates the
    /// detector. For a two-sided detector composed from independent upper and
    /// lower [`SpotDetector`] instances, compute both scores and take their
    /// maximum without changing either detector's existing update policy.
    ///
    /// # Invalid state
    ///
    /// Returns `NaN` for a `NaN` input or when the detector has not been fitted
    /// with valid GPD parameters. Infinite values in the modeled direction
    /// receive a score of `1`.
    pub fn anomaly_score(&self, value: f64) -> f64 {
        if value.is_nan() || self.n == 0 || self.excess_threshold.is_nan() {
            return f64::NAN;
        }

        let excess = self.up_down * (value - self.excess_threshold);
        self.tail.cdf(excess)
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
    pub fn config(&self) -> Option<SpotConfig> {
        Some(SpotConfig {
            q: self.q,
            low_tail: self.low,
            discard_anomalies: self.discard_anomalies,
            level: self.level,
            max_excess: self.tail.peaks().container().capacity(),
        })
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

    /// Get the configured estimator policy.
    pub fn estimator(&self) -> SpotEstimator {
        self.estimator
    }

    /// Get the configured initial threshold strategy.
    pub fn initial_threshold(&self) -> SpotInitialThreshold {
        self.initial_threshold
    }

    /// Get the configured streaming excess update condition.
    pub fn excess_update(&self) -> SpotExcessUpdate {
        self.excess_update
    }

    /// Reset the detector's internal state, keeping the configuration and the
    /// backing buffer. After calling this, [`fit`](Self::fit) must be called
    /// again before further [`step`](Self::step) calls.
    ///
    /// This mirrors the `spot_reset` C API exposed by the FFI wrapper crate.
    pub fn reset(&mut self) {
        self.anomaly_threshold = f64::NAN;
        self.excess_threshold = f64::NAN;
        self.nt = 0;
        self.n = 0;
        self.tail.reset();
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

    fn initial_quantile(&self, level: f64, data: &[f64]) -> f64 {
        match self.initial_threshold {
            SpotInitialThreshold::P2 => p2_quantile(level, data),
            SpotInitialThreshold::Empirical => empirical_quantile(level, data),
        }
    }

    fn should_update_excess(&self, excess: f64) -> bool {
        match self.excess_update {
            SpotExcessUpdate::GreaterOrEqual => excess >= 0.0,
            SpotExcessUpdate::Greater => excess > 0.0,
        }
    }
}

fn empirical_quantile(level: f64, data: &[f64]) -> f64 {
    let mut sorted: Vec<f64> = data
        .iter()
        .copied()
        .filter(|value| value.is_finite())
        .collect();
    if sorted.is_empty() {
        return f64::NAN;
    }

    sorted.sort_by(|a, b| a.total_cmp(b));
    let index = ((level - level.floor()) * sorted.len() as f64) as usize;
    sorted[index.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_spot_creation_valid_config() {
        let config = SpotConfig::default();
        let spot = SpotDetector::new(config).unwrap();

        assert_relative_eq!(spot.q, 0.0001);
        assert!(!spot.low);
        assert!(spot.discard_anomalies);
        assert_relative_eq!(spot.level, 0.998);
        assert!(spot.anomaly_threshold().is_nan());
        assert!(spot.excess_threshold().is_nan());
        assert_eq!(spot.n(), 0);
        assert_eq!(spot.nt(), 0);
        assert_eq!(spot.estimator(), SpotEstimator::Best);
        assert_eq!(spot.initial_threshold(), SpotInitialThreshold::P2);
        assert_eq!(spot.excess_update(), SpotExcessUpdate::GreaterOrEqual);
    }

    #[test]
    fn test_spot_creation_with_mom_estimator() {
        let config = SpotConfig {
            q: 0.001,
            level: 0.98,
            max_excess: 10_000,
            ..SpotConfig::default()
        };
        let mut spot = SpotDetector::new_with_full_options(
            config,
            SpotEstimator::Mom,
            SpotInitialThreshold::Empirical,
            SpotExcessUpdate::Greater,
        )
        .unwrap();

        let data: Vec<f64> = (0..1000)
            .map(|i| {
                let x = i as f64 * 0.1;
                x.sin() + (x * 0.37).cos() * 0.1 + 5.0
            })
            .collect();

        spot.fit(&data).unwrap();

        assert_eq!(spot.estimator(), SpotEstimator::Mom);
        assert_eq!(spot.initial_threshold(), SpotInitialThreshold::Empirical);
        assert_eq!(spot.excess_update(), SpotExcessUpdate::Greater);
        assert!(!spot.anomaly_threshold().is_nan());
        assert_eq!(spot.step(5.5).unwrap(), SpotStatus::Normal);
        assert_eq!(spot.step(100.0).unwrap(), SpotStatus::Anomaly);
    }

    #[test]
    fn test_empirical_quantile_matches_sorted_index() {
        let data = [5.0, 1.0, 3.0, 2.0, 4.0];

        assert_relative_eq!(empirical_quantile(0.0, &data), 1.0);
        assert_relative_eq!(empirical_quantile(0.5, &data), 3.0);
        assert_relative_eq!(empirical_quantile(0.98, &data), 5.0);
    }

    #[test]
    fn test_spot_invalid_level() {
        let config = SpotConfig {
            level: 1.5, // Invalid
            ..SpotConfig::default()
        };
        let result = SpotDetector::new(config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SpotError::LevelOutOfBounds);
    }

    #[test]
    fn test_spot_invalid_q() {
        let config = SpotConfig {
            q: 0.5, // Too high for level 0.998
            ..SpotConfig::default()
        };
        let result = SpotDetector::new(config);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SpotError::QOutOfBounds);
    }

    #[test]
    fn test_spot_fit_basic() {
        let config = SpotConfig::default();
        let mut spot = SpotDetector::new(config).unwrap();

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
        let mut spot = SpotDetector::new(config).unwrap();

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
        let mut spot = SpotDetector::new(config).unwrap();

        let result = spot.step(f64::NAN);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SpotError::DataIsNaN);
    }

    #[test]
    fn test_spot_observe_normal_advances_count_only() {
        let config = SpotConfig::default();
        let mut spot = SpotDetector::new(config).unwrap();

        let data: Vec<f64> = (0..1000).map(|i| (i as f64 / 1000.0) * 2.0 - 1.0).collect();
        spot.fit(&data).unwrap();
        let n = spot.n();
        let nt = spot.nt();
        let anomaly_threshold = spot.anomaly_threshold();
        let excess_threshold = spot.excess_threshold();

        spot.observe_normal();

        assert_eq!(spot.n(), n + 1);
        assert_eq!(spot.nt(), nt);
        assert_relative_eq!(spot.anomaly_threshold(), anomaly_threshold);
        assert_relative_eq!(spot.excess_threshold(), excess_threshold);
    }

    #[test]
    fn test_spot_reset_returns_to_pristine_state() {
        let config = SpotConfig::default();
        let mut spot = SpotDetector::new(config.clone()).unwrap();

        let data: Vec<f64> = (0..1000).map(|i| (i as f64 / 1000.0) * 2.0 - 1.0).collect();
        spot.fit(&data).unwrap();
        for v in &data {
            let _ = spot.step(*v).unwrap();
        }
        assert!(spot.n() > 0);
        assert!(!spot.anomaly_threshold().is_nan());

        spot.reset();

        // Looks like a freshly constructed detector.
        assert!(spot.anomaly_threshold().is_nan());
        assert!(spot.excess_threshold().is_nan());
        assert_eq!(spot.n(), 0);
        assert_eq!(spot.nt(), 0);
        assert_eq!(spot.tail_size(), 0);
        assert_eq!(spot.config(), Some(config.clone()));

        // Re-fit produces identical numbers to a fresh detector.
        let mut fresh = SpotDetector::new(config).unwrap();
        spot.fit(&data).unwrap();
        fresh.fit(&data).unwrap();
        assert_relative_eq!(spot.anomaly_threshold(), fresh.anomaly_threshold());
        assert_relative_eq!(spot.excess_threshold(), fresh.excess_threshold());
        assert_eq!(spot.nt(), fresh.nt());
        assert_eq!(spot.n(), fresh.n());
    }

    #[test]
    fn test_spot_reset_before_fit_is_noop_safe() {
        // Calling reset on a freshly constructed detector must not panic
        // and must leave the detector in the same observable state.
        let mut spot = SpotDetector::new(SpotConfig::default()).unwrap();
        spot.reset();
        assert!(spot.anomaly_threshold().is_nan());
        assert!(spot.excess_threshold().is_nan());
        assert_eq!(spot.n(), 0);
        assert_eq!(spot.nt(), 0);
        assert_eq!(spot.tail_size(), 0);

        // Fit still works normally afterwards.
        let data: Vec<f64> = (0..500).map(|i| (i as f64 / 500.0) * 2.0 - 1.0).collect();
        spot.fit(&data).unwrap();
        assert!(!spot.anomaly_threshold().is_nan());
    }

    #[test]
    fn test_spot_reset_is_idempotent() {
        let mut spot = SpotDetector::new(SpotConfig::default()).unwrap();
        let data: Vec<f64> = (0..500).map(|i| (i as f64 / 500.0) * 2.0 - 1.0).collect();
        spot.fit(&data).unwrap();
        for v in &data {
            let _ = spot.step(*v).unwrap();
        }

        spot.reset();
        let after_first_n = spot.n();
        let after_first_nt = spot.nt();
        let after_first_size = spot.tail_size();

        spot.reset();
        assert_eq!(spot.n(), after_first_n);
        assert_eq!(spot.nt(), after_first_nt);
        assert_eq!(spot.tail_size(), after_first_size);
        assert!(spot.anomaly_threshold().is_nan());
        assert!(spot.excess_threshold().is_nan());
    }

    #[test]
    fn test_spot_reset_then_fit_then_step_full_cycle() {
        // Full lifecycle: fit -> step -> reset -> fit again -> step again must
        // produce the same step classifications as a fresh detector running
        // the same fit+step sequence.
        let config = SpotConfig::default();
        let train: Vec<f64> = (0..1000).map(|i| (i as f64 / 1000.0) * 2.0 - 1.0).collect();
        let probe: Vec<f64> = (0..200).map(|i| (i as f64 / 100.0) - 1.0).collect();

        let mut reused = SpotDetector::new(config.clone()).unwrap();
        reused.fit(&train).unwrap();
        for v in &probe {
            let _ = reused.step(*v).unwrap();
        }
        reused.reset();
        reused.fit(&train).unwrap();
        let reused_classifications: Vec<SpotStatus> =
            probe.iter().map(|&v| reused.step(v).unwrap()).collect();

        let mut fresh = SpotDetector::new(config).unwrap();
        fresh.fit(&train).unwrap();
        let fresh_classifications: Vec<SpotStatus> =
            probe.iter().map(|&v| fresh.step(v).unwrap()).collect();

        assert_eq!(reused_classifications, fresh_classifications);
        assert_relative_eq!(reused.anomaly_threshold(), fresh.anomaly_threshold());
        assert_relative_eq!(reused.excess_threshold(), fresh.excess_threshold());
        assert_eq!(reused.nt(), fresh.nt());
        assert_eq!(reused.n(), fresh.n());
    }

    #[test]
    fn test_spot_repeated_fit_replaces_previous_tail() {
        let config = SpotConfig {
            level: 0.9,
            ..SpotConfig::default()
        };
        let train_a: Vec<f64> = (0..100).map(|i| i as f64).collect();
        let train_b: Vec<f64> = (0..50).map(|i| i as f64 * 0.5).collect();

        let mut reused = SpotDetector::new_with_options(
            config.clone(),
            SpotEstimator::Best,
            SpotInitialThreshold::Empirical,
        )
        .unwrap();
        reused.fit(&train_a).unwrap();
        reused.fit(&train_b).unwrap();

        let mut fresh = SpotDetector::new_with_options(
            config,
            SpotEstimator::Best,
            SpotInitialThreshold::Empirical,
        )
        .unwrap();
        fresh.fit(&train_b).unwrap();

        assert_eq!(reused.tail_size(), reused.nt());
        assert_eq!(reused.tail_size(), fresh.tail_size());
        assert_eq!(reused.nt(), fresh.nt());
        assert_eq!(reused.n(), fresh.n());
        assert_relative_eq!(reused.anomaly_threshold(), fresh.anomaly_threshold());
        assert_relative_eq!(reused.excess_threshold(), fresh.excess_threshold());
    }

    #[test]
    fn test_spot_low_tail() {
        let config = SpotConfig {
            low_tail: true,
            ..SpotConfig::default()
        };
        let spot = SpotDetector::new(config).unwrap();

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

        let spot = SpotDetector::new(original_config.clone()).unwrap();
        let retrieved_config = spot.config().unwrap();

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
        let mut spot = SpotDetector::new(config).unwrap();

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

    fn score_test_detector(low_tail: bool) -> SpotDetector {
        let config = SpotConfig {
            q: 0.001,
            low_tail,
            discard_anomalies: true,
            level: 0.9,
            max_excess: 200,
        };
        let mut spot = SpotDetector::new_with_options(
            config,
            SpotEstimator::Best,
            SpotInitialThreshold::Empirical,
        )
        .unwrap();
        let data: Vec<f64> = (0..1000)
            .map(|i| {
                let x = i as f64 * 0.031;
                x.sin() + 0.2 * (x * 0.37).cos()
            })
            .collect();
        spot.fit(&data).unwrap();
        spot
    }

    #[test]
    fn test_anomaly_score_requires_fitted_detector_and_valid_value() {
        let spot = SpotDetector::new(SpotConfig::default()).unwrap();
        assert!(spot.anomaly_score(1.0).is_nan());
        assert!(spot.anomaly_score(f64::NAN).is_nan());
    }

    #[test]
    fn test_anomaly_score_range_and_monotonicity_for_both_tails() {
        for low_tail in [false, true] {
            let spot = score_test_detector(low_tail);
            let direction = if low_tail { -1.0 } else { 1.0 };
            let threshold = spot.excess_threshold();
            let anomaly_distance = direction * (spot.anomaly_threshold() - threshold);

            assert_eq!(spot.anomaly_score(threshold - direction), 0.0);

            let mut previous = 0.0;
            for scale in [0.0, 0.25, 0.5, 1.0, 2.0] {
                let value = threshold + direction * anomaly_distance * scale;
                let score = spot.anomaly_score(value);

                assert!((0.0..=1.0).contains(&score));
                assert!(score >= previous, "low_tail={low_tail}, value={value}");
                previous = score;
            }
        }
    }

    #[test]
    fn test_anomaly_score_matches_probability_conditioned_on_tail() {
        for low_tail in [false, true] {
            let spot = score_test_detector(low_tail);
            let t = spot.excess_threshold();
            let z = spot.anomaly_threshold();
            let value = (t + z) / 2.0;
            let tail_fraction = spot.nt() as f64 / spot.n() as f64;

            assert_relative_eq!(
                spot.anomaly_score(value),
                1.0 - spot.probability(value) / tail_fraction,
                epsilon = 1e-8
            );
        }
    }

    #[test]
    fn test_anomaly_score_is_read_only() {
        let spot = score_test_detector(false);
        let before = (
            spot.n(),
            spot.nt(),
            spot.anomaly_threshold(),
            spot.excess_threshold(),
            spot.tail_parameters(),
            spot.peaks_data(),
        );

        let _ = spot.anomaly_score(spot.anomaly_threshold() * 2.0);

        assert_eq!(spot.n(), before.0);
        assert_eq!(spot.nt(), before.1);
        assert_eq!(spot.anomaly_threshold(), before.2);
        assert_eq!(spot.excess_threshold(), before.3);
        assert_eq!(spot.tail_parameters(), before.4);
        assert_eq!(spot.peaks_data(), before.5);
    }

    #[test]
    fn test_upper_and_lower_anomaly_scores_are_mirror_symmetric() {
        let upper = score_test_detector(false);

        let config = SpotConfig {
            q: 0.001,
            low_tail: true,
            discard_anomalies: true,
            level: 0.9,
            max_excess: 200,
        };
        let mut lower = SpotDetector::new_with_options(
            config,
            SpotEstimator::Best,
            SpotInitialThreshold::Empirical,
        )
        .unwrap();
        let mirrored_data: Vec<f64> = (0..1000)
            .map(|i| {
                let x = i as f64 * 0.031;
                -(x.sin() + 0.2 * (x * 0.37).cos())
            })
            .collect();
        lower.fit(&mirrored_data).unwrap();

        for value in [
            upper.excess_threshold(),
            (upper.excess_threshold() + upper.anomaly_threshold()) / 2.0,
            upper.anomaly_threshold(),
            upper.anomaly_threshold() * 1.5,
        ] {
            assert_relative_eq!(
                upper.anomaly_score(value),
                lower.anomaly_score(-value),
                epsilon = 1e-12
            );
        }
    }

    #[test]
    fn test_spot_excess_detection() {
        let config = SpotConfig {
            level: 0.9, // Lower level for easier testing
            ..SpotConfig::default()
        };
        let mut spot = SpotDetector::new(config).unwrap();

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
