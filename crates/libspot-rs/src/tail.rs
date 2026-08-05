//! Tail structure for GPD modeling
//!
//! This module implements the Tail structure that models the tail of a distribution
//! using Generalized Pareto Distribution (GPD) parameters.

use crate::config::SpotEstimator;
use crate::error::SpotResult;

use crate::estimator::{grimshaw_estimator, mom_estimator, mom_sample_variance_estimator};
use crate::math::{xexp, xlog, xpow};
use crate::peaks::Peaks;

/// Structure that embeds GPD parameters (GPD tail actually)
///
/// # Serialization
///
/// When the `serde` feature is enabled, this struct can be serialized and deserialized.
/// This allows saving and restoring the GPD tail model parameters.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Tail {
    /// GPD gamma parameter
    #[cfg_attr(feature = "serde", serde(with = "crate::ser::nan_safe_f64"))]
    gamma: f64,
    /// GPD sigma parameter
    #[cfg_attr(feature = "serde", serde(with = "crate::ser::nan_safe_f64"))]
    sigma: f64,
    /// Underlying Peaks structure
    peaks: Peaks,
}

impl Tail {
    /// Initialize a new Tail structure with the given size
    pub fn new(size: usize) -> SpotResult<Self> {
        Ok(Self {
            gamma: f64::NAN,
            sigma: f64::NAN,
            peaks: Peaks::new(size)?,
        })
    }

    /// Add a new data point into the tail
    pub fn push(&mut self, x: f64) {
        self.peaks.push(x);
    }

    /// Reset the tail to its initial state, keeping the allocated buffer.
    pub(crate) fn reset(&mut self) {
        self.gamma = f64::NAN;
        self.sigma = f64::NAN;
        self.peaks.reset();
    }

    /// Fit the GPD parameters using the default estimator policy.
    ///
    /// Returns the selected fit log-likelihood.
    pub fn fit(&mut self) -> f64 {
        self.fit_with(SpotEstimator::Best)
    }

    /// Fit the GPD parameters using a selected estimator policy.
    ///
    /// Returns the selected fit log-likelihood.
    pub fn fit_with(&mut self, estimator: SpotEstimator) -> f64 {
        if self.peaks.size() == 0 {
            return f64::NAN;
        }

        match estimator {
            SpotEstimator::Best => self.fit_best(),
            SpotEstimator::Mom => self.fit_mom(),
        }
    }

    fn fit_mom(&mut self) -> f64 {
        let (gamma, sigma, llhood) = mom_sample_variance_estimator(&self.peaks);
        self.gamma = gamma;
        self.sigma = sigma;
        llhood
    }

    fn fit_best(&mut self) -> f64 {
        // Match C implementation exactly: try each estimator and pick best
        let mut max_llhood = f64::NAN;
        let mut tmp_gamma;
        let mut tmp_sigma;

        // Try MoM estimator first (index 0 in C)
        let llhood = {
            let (gamma, sigma, llhood) = mom_estimator(&self.peaks);
            tmp_gamma = gamma;
            tmp_sigma = sigma;
            llhood
        };

        if max_llhood.is_nan() || llhood > max_llhood {
            max_llhood = llhood;
            self.gamma = tmp_gamma;
            self.sigma = tmp_sigma;
        }

        // Try Grimshaw estimator (index 1 in C)
        let llhood = {
            let (gamma, sigma, llhood) = grimshaw_estimator(&self.peaks);
            tmp_gamma = gamma;
            tmp_sigma = sigma;
            llhood
        };

        if max_llhood.is_nan() || llhood > max_llhood {
            // Back to original logic
            max_llhood = llhood;
            self.gamma = tmp_gamma;
            self.sigma = tmp_sigma;
        }

        max_llhood
    }

    /// Compute the probability P(X > z) = p given the tail threshold difference d = z - t
    pub fn probability(&self, s: f64, d: f64) -> f64 {
        if self.gamma.is_nan() || self.sigma.is_nan() || self.sigma <= 0.0 {
            return f64::NAN;
        }

        // Use exact equality check like C implementation (no tolerance)
        if self.gamma == 0.0 {
            s * xexp(-d / self.sigma)
        } else {
            let r = d * (self.gamma / self.sigma);
            s * xpow(1.0 + r, -1.0 / self.gamma)
        }
    }

    /// Conditional GPD CDF for an excess, evaluated through the cumulative
    /// hazard with `ln_1p` and `exp_m1` for precision near the tail threshold.
    pub(crate) fn cdf(&self, d: f64) -> f64 {
        if !self.gamma.is_finite() || !self.sigma.is_finite() || self.sigma <= 0.0 {
            return f64::NAN;
        }
        if d.is_nan() {
            return f64::NAN;
        }
        if d <= 0.0 {
            return 0.0;
        }

        if self.gamma < 0.0 && d >= -self.sigma / self.gamma {
            return 1.0;
        }

        // Unlike the C-compatible probability path, scoring uses ln_1p/exp_m1
        // to avoid cancellation near the tail threshold.
        let hazard = if self.gamma == 0.0 {
            d / self.sigma
        } else {
            (self.gamma * d / self.sigma).ln_1p() / self.gamma
        };

        if hazard.is_nan() || hazard < 0.0 {
            return f64::NAN;
        }

        // 1 - exp(-hazard), evaluated accurately when hazard is close to zero.
        (-(-hazard).exp_m1()).clamp(0.0, 1.0)
    }

    /// Compute the extreme quantile for given probability q
    /// s is the ratio Nt/n (an estimator of P(X>t) = 1-F(t))
    /// q is the desired low probability
    pub fn quantile(&self, s: f64, q: f64) -> f64 {
        if self.gamma.is_nan() || self.sigma.is_nan() || self.sigma <= 0.0 {
            return f64::NAN;
        }

        let r = q / s;
        // Use exact equality check like C implementation (no tolerance)
        if self.gamma == 0.0 {
            -self.sigma * xlog(r)
        } else {
            (self.sigma / self.gamma) * (xpow(r, -self.gamma) - 1.0)
        }
    }

    /// Get the current gamma parameter
    pub fn gamma(&self) -> f64 {
        self.gamma
    }

    /// Get the current sigma parameter
    pub fn sigma(&self) -> f64 {
        self.sigma
    }

    /// Get the current size of the tail data
    pub fn size(&self) -> usize {
        self.peaks.size()
    }

    /// Get access to the underlying peaks structure
    pub fn peaks(&self) -> &Peaks {
        &self.peaks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SpotError;
    use approx::assert_relative_eq;

    #[test]
    fn test_tail_reset_clears_gpd_params_and_peaks() {
        let mut tail = Tail::new(50).unwrap();
        for i in 0..40 {
            tail.push(0.1 + i as f64 * 0.05);
        }
        let _ = tail.fit();
        assert!(tail.size() > 0);
        // gamma/sigma may be NaN if the fit fails on this trivial input,
        // so we only assert they're cleared post-reset, not pre-reset.

        tail.reset();

        assert_eq!(tail.size(), 0);
        assert!(tail.gamma().is_nan());
        assert!(tail.sigma().is_nan());
    }

    #[test]
    fn test_tail_creation() {
        let tail = Tail::new(10).unwrap();
        assert_eq!(tail.size(), 0);
        assert!(tail.gamma().is_nan());
        assert!(tail.sigma().is_nan());
    }

    #[test]
    fn test_tail_zero_size() {
        let result = Tail::new(0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SpotError::MemoryAllocationFailed);
    }

    #[test]
    fn test_tail_push() {
        let mut tail = Tail::new(5).unwrap();

        tail.push(1.0);
        assert_eq!(tail.size(), 1);

        tail.push(2.0);
        tail.push(3.0);
        assert_eq!(tail.size(), 3);
    }

    #[test]
    fn test_tail_fit_empty() {
        let mut tail = Tail::new(5).unwrap();
        let llhood = tail.fit();
        assert!(llhood.is_nan());
        assert!(tail.gamma().is_nan());
        assert!(tail.sigma().is_nan());
    }

    #[test]
    fn test_tail_fit_with_data() {
        let mut tail = Tail::new(10).unwrap();

        // Add some sample data
        for value in [1.0, 1.5, 2.0, 2.5, 3.0, 1.2, 1.8, 2.2] {
            tail.push(value);
        }

        let llhood = tail.fit();
        assert!(!llhood.is_nan());
        assert!(llhood.is_finite());

        // Parameters should be fitted
        assert!(!tail.gamma().is_nan());
        assert!(!tail.sigma().is_nan());
        assert!(tail.sigma() > 0.0); // Sigma should be positive
    }

    #[test]
    fn test_tail_fit_with_mom_estimator() {
        let mut tail = Tail::new(10).unwrap();

        for value in [1.0, 1.5, 2.0, 2.5, 3.0, 1.2, 1.8, 2.2] {
            tail.push(value);
        }

        let llhood = tail.fit_with(SpotEstimator::Mom);
        assert!(!llhood.is_nan());
        assert!(!tail.gamma().is_nan());
        assert!(!tail.sigma().is_nan());
        assert!(tail.sigma() > 0.0);
    }

    #[test]
    fn test_tail_quantile_gamma_zero() {
        let mut tail = Tail::new(10).unwrap();

        // Manually set parameters for testing
        tail.gamma = 0.0;
        tail.sigma = 1.0;

        let q = tail.quantile(0.1, 0.01);
        assert!(!q.is_nan());
        assert!(q > 0.0); // Should be positive for low probability
    }

    #[test]
    fn test_tail_quantile_gamma_nonzero() {
        let mut tail = Tail::new(10).unwrap();

        // Manually set parameters for testing
        tail.gamma = 0.1;
        tail.sigma = 1.0;

        let q = tail.quantile(0.1, 0.01);
        assert!(!q.is_nan());
        assert!(q.is_finite());
    }

    #[test]
    fn test_tail_probability_gamma_zero() {
        let mut tail = Tail::new(10).unwrap();

        // Manually set parameters for testing
        tail.gamma = 0.0;
        tail.sigma = 1.0;

        let p = tail.probability(0.1, 2.0);
        assert!(!p.is_nan());
        assert!((0.0..=0.1).contains(&p));
    }

    #[test]
    fn test_tail_probability_gamma_nonzero() {
        let mut tail = Tail::new(10).unwrap();

        // Manually set parameters for testing
        tail.gamma = 0.1;
        tail.sigma = 1.0;

        let p = tail.probability(0.1, 2.0);
        assert!(!p.is_nan());
        assert!(p >= 0.0);
    }

    #[test]
    fn test_tail_cdf_exponential_tail() {
        let mut tail = Tail::new(10).unwrap();
        tail.gamma = 0.0;
        tail.sigma = 2.0;

        assert_eq!(tail.cdf(-1.0), 0.0);
        assert_eq!(tail.cdf(0.0), 0.0);
        assert_relative_eq!(tail.cdf(2.0), 1.0 - (-1.0_f64).exp(), epsilon = 1e-15);
        assert_eq!(tail.cdf(f64::INFINITY), 1.0);
    }

    #[test]
    fn test_tail_cdf_heavy_tail() {
        let mut tail = Tail::new(10).unwrap();
        tail.gamma = 0.5;
        tail.sigma = 2.0;

        let d = 4.0;
        let expected = 1.0 - (1.0_f64 + tail.gamma * d / tail.sigma).powf(-1.0 / tail.gamma);
        assert_relative_eq!(tail.cdf(d), expected, epsilon = 1e-15);
    }

    #[test]
    fn test_tail_cdf_bounded_tail_reaches_one_at_endpoint() {
        let mut tail = Tail::new(10).unwrap();
        tail.gamma = -0.5;
        tail.sigma = 2.0;

        let endpoint = -tail.sigma / tail.gamma;
        assert!(tail.cdf(endpoint - 1e-6) < 1.0);
        assert_eq!(tail.cdf(endpoint), 1.0);
        assert_eq!(tail.cdf(endpoint + 1.0), 1.0);
        assert_eq!(tail.cdf(f64::INFINITY), 1.0);
    }

    #[test]
    fn test_tail_cdf_is_monotone_for_all_tail_shapes() {
        for gamma in [-0.25, 0.0, 0.5] {
            let mut tail = Tail::new(10).unwrap();
            tail.gamma = gamma;
            tail.sigma = 2.0;

            let upper = if gamma < 0.0 {
                -tail.sigma / gamma
            } else {
                20.0
            };
            let mut previous = 0.0;
            for i in 0..=100 {
                let d = upper * i as f64 / 100.0;
                let score = tail.cdf(d);
                assert!((0.0..=1.0).contains(&score));
                assert!(score >= previous, "gamma={gamma}, d={d}");
                previous = score;
            }
        }
    }

    #[test]
    fn test_tail_cdf_matches_conditional_survival() {
        for gamma in [-0.25, 0.0, 0.5] {
            let mut tail = Tail::new(10).unwrap();
            tail.gamma = gamma;
            tail.sigma = 2.0;

            for d in [0.25, 0.5, 1.0, 2.0] {
                let survival = tail.probability(1.0, d);
                assert_relative_eq!(tail.cdf(d), 1.0 - survival, epsilon = 1e-14);
            }
        }
    }

    #[test]
    fn test_tail_cdf_rejects_invalid_parameters_and_nan() {
        let mut tail = Tail::new(10).unwrap();
        assert!(tail.cdf(1.0).is_nan());

        tail.gamma = 0.1;
        tail.sigma = 0.0;
        assert!(tail.cdf(1.0).is_nan());

        tail.sigma = 1.0;
        assert!(tail.cdf(f64::NAN).is_nan());
    }

    #[test]
    fn test_tail_invalid_parameters() {
        let mut tail = Tail::new(10).unwrap();

        // Test with invalid sigma
        tail.gamma = 0.1;
        tail.sigma = 0.0;

        let q = tail.quantile(0.1, 0.01);
        assert!(q.is_nan());

        let p = tail.probability(0.1, 2.0);
        assert!(p.is_nan());
    }

    #[test]
    fn test_tail_consistency() {
        let mut tail = Tail::new(10).unwrap();

        // Add some data and fit
        for value in [0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0] {
            tail.push(value);
        }

        let _llhood = tail.fit();

        // Test that quantile and probability are somewhat consistent
        let s = 0.1;
        let q = 0.01;
        let quantile_val = tail.quantile(s, q);

        if !quantile_val.is_nan() && quantile_val.is_finite() {
            let prob_val = tail.probability(s, quantile_val);
            if !prob_val.is_nan() && prob_val.is_finite() {
                // The probability should be approximately q
                // Allow for some numerical error
                assert!((prob_val - q).abs() < q * 0.1 || prob_val < q * 2.0);
            }
        }
    }
}
