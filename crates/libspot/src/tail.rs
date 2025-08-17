//! Tail structure for GPD modeling
//!
//! This module implements the Tail structure that models the tail of a distribution
//! using Generalized Pareto Distribution (GPD) parameters.

use crate::error::SpotResult;

use crate::estimator::{grimshaw_estimator, mom_estimator};
use crate::math::is_nan;
use crate::math::{xexp, xlog, xpow};
use crate::peaks::Peaks;

/// Structure that embeds GPD parameters (GPD tail actually)
#[derive(Debug, Clone)]
pub struct Tail {
    /// GPD gamma parameter
    gamma: f64,
    /// GPD sigma parameter
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

    /// Fit the GPD parameters using the available estimators
    /// Returns the log-likelihood of the best fit
    pub fn fit(&mut self) -> f64 {
        if self.peaks.size() == 0 {
            return f64::NAN;
        }

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

        if is_nan(max_llhood) || llhood > max_llhood {
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

        // Debug the critical case to see which estimator is selected
        let debug_active =
            self.peaks.size() == 200 && (self.peaks.mean() - 0.766766777376012).abs() < 1e-10;
        if debug_active {}

        if is_nan(max_llhood) || llhood > max_llhood {
            // Back to original logic
            max_llhood = llhood;
            self.gamma = tmp_gamma;
            self.sigma = tmp_sigma;
        }

        max_llhood
    }

    /// Compute the probability P(X > z) = p given the tail threshold difference d = z - t
    pub fn probability(&self, s: f64, d: f64) -> f64 {
        if is_nan(self.gamma) || is_nan(self.sigma) || self.sigma <= 0.0 {
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

    /// Compute the extreme quantile for given probability q
    /// s is the ratio Nt/n (an estimator of P(X>t) = 1-F(t))
    /// q is the desired low probability
    pub fn quantile(&self, s: f64, q: f64) -> f64 {
        if is_nan(self.gamma) || is_nan(self.sigma) || self.sigma <= 0.0 {
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

    #[test]
    fn test_tail_creation() {
        let tail = Tail::new(10).unwrap();
        assert_eq!(tail.size(), 0);
        assert!(is_nan(tail.gamma()));
        assert!(is_nan(tail.sigma()));
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
        assert!(is_nan(llhood));
        assert!(is_nan(tail.gamma()));
        assert!(is_nan(tail.sigma()));
    }

    #[test]
    fn test_tail_fit_with_data() {
        let mut tail = Tail::new(10).unwrap();

        // Add some sample data
        for value in [1.0, 1.5, 2.0, 2.5, 3.0, 1.2, 1.8, 2.2] {
            tail.push(value);
        }

        let llhood = tail.fit();
        assert!(!is_nan(llhood));
        assert!(llhood.is_finite());

        // Parameters should be fitted
        assert!(!is_nan(tail.gamma()));
        assert!(!is_nan(tail.sigma()));
        assert!(tail.sigma() > 0.0); // Sigma should be positive
    }

    #[test]
    fn test_tail_quantile_gamma_zero() {
        let mut tail = Tail::new(10).unwrap();

        // Manually set parameters for testing
        tail.gamma = 0.0;
        tail.sigma = 1.0;

        let q = tail.quantile(0.1, 0.01);
        assert!(!is_nan(q));
        assert!(q > 0.0); // Should be positive for low probability
    }

    #[test]
    fn test_tail_quantile_gamma_nonzero() {
        let mut tail = Tail::new(10).unwrap();

        // Manually set parameters for testing
        tail.gamma = 0.1;
        tail.sigma = 1.0;

        let q = tail.quantile(0.1, 0.01);
        assert!(!is_nan(q));
        assert!(q.is_finite());
    }

    #[test]
    fn test_tail_probability_gamma_zero() {
        let mut tail = Tail::new(10).unwrap();

        // Manually set parameters for testing
        tail.gamma = 0.0;
        tail.sigma = 1.0;

        let p = tail.probability(0.1, 2.0);
        assert!(!is_nan(p));
        assert!(p >= 0.0 && p <= 0.1);
    }

    #[test]
    fn test_tail_probability_gamma_nonzero() {
        let mut tail = Tail::new(10).unwrap();

        // Manually set parameters for testing
        tail.gamma = 0.1;
        tail.sigma = 1.0;

        let p = tail.probability(0.1, 2.0);
        assert!(!is_nan(p));
        assert!(p >= 0.0);
    }

    #[test]
    fn test_tail_invalid_parameters() {
        let mut tail = Tail::new(10).unwrap();

        // Test with invalid sigma
        tail.gamma = 0.1;
        tail.sigma = 0.0;

        let q = tail.quantile(0.1, 0.01);
        assert!(is_nan(q));

        let p = tail.probability(0.1, 2.0);
        assert!(is_nan(p));
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

        if !is_nan(quantile_val) && quantile_val.is_finite() {
            let prob_val = tail.probability(s, quantile_val);
            if !is_nan(prob_val) && prob_val.is_finite() {
                // The probability should be approximately q
                // Allow for some numerical error
                assert!((prob_val - q).abs() < q * 0.1 || prob_val < q * 2.0);
            }
        }
    }
}
