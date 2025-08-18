//! Peaks structure for computing statistics over peak data
//!
//! This module implements the Peaks structure that computes statistics
//! about peaks data using an underlying Ubend circular buffer.

use crate::error::SpotResult;

use crate::ubend::Ubend;

/// Structure that computes stats about the peaks
#[derive(Debug, Clone)]
pub struct Peaks {
    /// Sum of the elements
    e: f64,
    /// Sum of the square of the elements
    e2: f64,
    /// Minimum of the elements
    min: f64,
    /// Maximum of the elements
    max: f64,
    /// Underlying data container
    container: Ubend,
}

impl Peaks {
    /// Initialize a new Peaks structure with the given size
    pub fn new(size: usize) -> SpotResult<Self> {
        Ok(Self {
            e: 0.0,
            e2: 0.0,
            min: f64::NAN,
            max: f64::NAN,
            container: Ubend::new(size)?,
        })
    }

    /// Get the current size of the peaks container
    pub fn size(&self) -> usize {
        self.container.size()
    }

    /// Add a new data point into the peaks
    pub fn push(&mut self, x: f64) {
        let erased = self.container.push(x);
        let size = self.size();

        // Increment the stats
        self.e += x;
        self.e2 += x * x;

        // First we update the stats with the value of x
        if size == 1 || x < self.min {
            self.min = x;
        }
        if size == 1 || x > self.max {
            self.max = x;
        }

        // Then we treat the case where a data has been erased
        // In this case we must update the accumulators and possibly update the min/max
        if !erased.is_nan() {
            self.e -= erased;
            self.e2 -= erased * erased;
            if (erased <= self.min) || (erased >= self.max) {
                // Here we have to loop in the container to ensure having
                // the right stats (in particular min and max). However, we
                // also update e and e2 (the in/decrements may create precision errors)
                self.update_stats();
            }
        }
    }

    /// Compute the mean of the elements
    pub fn mean(&self) -> f64 {
        let size = self.size();
        if size == 0 {
            f64::NAN
        } else {
            self.e / (size as f64)
        }
    }

    /// Compute the variance of the elements
    pub fn variance(&self) -> f64 {
        let size = self.size();
        if size == 0 {
            f64::NAN
        } else {
            let size_f = size as f64;
            let mean = self.e / size_f;
            (self.e2 / size_f) - (mean * mean)
        }
    }

    /// Get the minimum value
    pub fn min(&self) -> f64 {
        self.min
    }

    /// Get the maximum value
    pub fn max(&self) -> f64 {
        self.max
    }

    /// Get the sum of elements
    pub fn sum(&self) -> f64 {
        self.e
    }

    /// Get the sum of squares
    pub fn sum_squares(&self) -> f64 {
        self.e2
    }

    /// Get access to the underlying container
    pub fn container(&self) -> &Ubend {
        &self.container
    }

    /// Update all statistics by iterating through the container
    /// This is called when we need to recompute min/max after an erasure
    fn update_stats(&mut self) {
        // Reset min and max
        self.min = f64::NAN;
        self.max = f64::NAN;
        // Reset accumulators
        self.e = 0.0;
        self.e2 = 0.0;

        let max_iteration = self.container.size();

        for i in 0..max_iteration {
            // Direct access to container data (matches C implementation)
            let value = self.container.raw_data()[i];
            self.e += value;
            self.e2 += value * value;

            if self.min.is_nan() || (value < self.min) {
                self.min = value;
            }
            if self.max.is_nan() || (value > self.max) {
                self.max = value;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SpotError;
    use approx::assert_relative_eq;

    #[test]
    fn test_peaks_creation() {
        let peaks = Peaks::new(5).unwrap();
        assert_eq!(peaks.size(), 0);
        assert_relative_eq!(peaks.sum(), 0.0);
        assert_relative_eq!(peaks.sum_squares(), 0.0);
        assert!(peaks.min().is_nan());
        assert!(peaks.max().is_nan());
        assert!(peaks.mean().is_nan());
        assert!(peaks.variance().is_nan());
    }

    #[test]
    fn test_peaks_zero_size() {
        let result = Peaks::new(0);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), SpotError::MemoryAllocationFailed);
    }

    #[test]
    fn test_peaks_single_element() {
        let mut peaks = Peaks::new(3).unwrap();

        peaks.push(5.0);
        assert_eq!(peaks.size(), 1);
        assert_relative_eq!(peaks.sum(), 5.0);
        assert_relative_eq!(peaks.sum_squares(), 25.0);
        assert_relative_eq!(peaks.min(), 5.0);
        assert_relative_eq!(peaks.max(), 5.0);
        assert_relative_eq!(peaks.mean(), 5.0);
        assert_relative_eq!(peaks.variance(), 0.0);
    }

    #[test]
    fn test_peaks_multiple_elements() {
        let mut peaks = Peaks::new(5).unwrap();

        peaks.push(1.0);
        peaks.push(2.0);
        peaks.push(3.0);

        assert_eq!(peaks.size(), 3);
        assert_relative_eq!(peaks.sum(), 6.0);
        assert_relative_eq!(peaks.sum_squares(), 14.0);
        assert_relative_eq!(peaks.min(), 1.0);
        assert_relative_eq!(peaks.max(), 3.0);
        assert_relative_eq!(peaks.mean(), 2.0);

        // Variance = E[X²] - (E[X])² = 14/3 - 4 = 14/3 - 12/3 = 2/3
        assert_relative_eq!(peaks.variance(), 2.0 / 3.0, epsilon = 1e-14);
    }

    #[test]
    fn test_peaks_overflow_and_min_max_update() {
        let mut peaks = Peaks::new(3).unwrap();

        // Fill with 1, 2, 3
        peaks.push(1.0); // min=1, max=1
        peaks.push(2.0); // min=1, max=2
        peaks.push(3.0); // min=1, max=3

        assert_relative_eq!(peaks.min(), 1.0);
        assert_relative_eq!(peaks.max(), 3.0);

        // Add 0.5, which should erase 1.0 and become new minimum
        peaks.push(0.5); // should erase 1.0, so we have [2, 3, 0.5]

        assert_eq!(peaks.size(), 3);
        assert_relative_eq!(peaks.min(), 0.5);
        assert_relative_eq!(peaks.max(), 3.0);
        assert_relative_eq!(peaks.sum(), 5.5);

        // Add 4.0, which should erase 2.0 and become new maximum
        peaks.push(4.0); // should erase 2.0, so we have [3, 0.5, 4.0]

        assert_relative_eq!(peaks.min(), 0.5);
        assert_relative_eq!(peaks.max(), 4.0);
        assert_relative_eq!(peaks.sum(), 7.5);
    }

    #[test]
    fn test_peaks_stats_after_min_erasure() {
        let mut peaks = Peaks::new(3).unwrap();

        // Add values where the minimum will be erased
        peaks.push(2.0);
        peaks.push(1.0); // This is the minimum
        peaks.push(3.0);

        assert_relative_eq!(peaks.min(), 1.0);
        assert_relative_eq!(peaks.max(), 3.0);

        // Add a value that will force erasure of the minimum
        peaks.push(2.5); // This should erase 2.0, leaving [1, 3, 2.5]

        assert_relative_eq!(peaks.min(), 1.0); // Still 1.0
        assert_relative_eq!(peaks.max(), 3.0); // Still 3.0

        // Add another value that will erase the minimum
        peaks.push(2.7); // This should erase 1.0, leaving [3, 2.5, 2.7]

        // Now the minimum should be recalculated
        assert_relative_eq!(peaks.min(), 2.5);
        assert_relative_eq!(peaks.max(), 3.0);
    }

    #[test]
    fn test_peaks_stats_after_max_erasure() {
        let mut peaks = Peaks::new(3).unwrap();

        // Add values where the maximum will be erased
        peaks.push(1.0);
        peaks.push(3.0); // This is the maximum
        peaks.push(2.0);

        assert_relative_eq!(peaks.min(), 1.0);
        assert_relative_eq!(peaks.max(), 3.0);

        // Add values that will eventually force erasure of the maximum
        peaks.push(1.5); // This should erase 1.0, leaving [3, 2, 1.5]
        peaks.push(1.7); // This should erase 3.0, leaving [2, 1.5, 1.7]

        // Now the maximum should be recalculated
        assert_relative_eq!(peaks.min(), 1.5);
        assert_relative_eq!(peaks.max(), 2.0);
    }
}
