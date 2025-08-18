use std::mem::MaybeUninit;
use std::os::raw::c_ulong;

use crate::config::SpotConfig;
use crate::error::{SpotError, SpotResult};
use crate::ffi::{self, SpotRaw};
use crate::status::SpotStatus;

/// A SPOT detector for streaming anomaly detection
#[derive(Debug)]
pub struct SpotDetector {
    raw: MaybeUninit<SpotRaw>,
    initialized: bool,
}

impl SpotDetector {
    /// Create a new SPOT detector with the given configuration
    pub fn new(config: SpotConfig) -> SpotResult<Self> {
        // Initialize allocators
        unsafe {
            ffi::set_allocators(libc::malloc, libc::free);
        }

        let mut detector = SpotDetector {
            raw: MaybeUninit::uninit(),
            initialized: false,
        };

        unsafe {
            let status = ffi::spot_init(
                detector.raw.as_mut_ptr(),
                config.q,
                if config.low_tail { 1 } else { 0 },
                if config.discard_anomalies { 1 } else { 0 },
                config.level,
                config.max_excess,
            );

            if status < 0 {
                return Err(SpotError::from_code(status));
            }
        }

        detector.initialized = true;
        Ok(detector)
    }

    /// Fit the model using initial training data
    pub fn fit(&mut self, data: &[f64]) -> SpotResult<()> {
        if !self.initialized {
            return Err(SpotError::NotInitialized);
        }

        unsafe {
            let status = ffi::spot_fit(self.raw.as_mut_ptr(), data.as_ptr(), data.len() as c_ulong);

            if status < 0 {
                return Err(SpotError::from_code(status));
            }
        }

        Ok(())
    }

    /// Process a single data point and return its classification
    pub fn step(&mut self, value: f64) -> SpotResult<SpotStatus> {
        if !self.initialized {
            return Err(SpotError::NotInitialized);
        }

        unsafe {
            let status = ffi::spot_step(self.raw.as_mut_ptr(), value);
            if status < 0 {
                return Err(SpotError::from_code(status));
            }
            Ok(SpotStatus::from(status))
        }
    }

    /// Get the quantile for a given probability
    pub fn quantile(&self, q: f64) -> f64 {
        if !self.initialized {
            return f64::NAN;
        }

        unsafe { ffi::spot_quantile(self.raw.as_ptr(), q) }
    }

    /// Get the current anomaly threshold
    pub fn anomaly_threshold(&self) -> f64 {
        if !self.initialized {
            return f64::NAN;
        }

        unsafe {
            let spot_ref = &*self.raw.as_ptr();
            spot_ref.anomaly_threshold
        }
    }

    /// Get the current excess threshold
    pub fn excess_threshold(&self) -> f64 {
        if !self.initialized {
            return f64::NAN;
        }

        unsafe {
            let spot_ref = &*self.raw.as_ptr();
            spot_ref.excess_threshold
        }
    }

    /// Get the current configuration parameters
    pub fn config(&self) -> Option<SpotConfig> {
        if !self.initialized {
            return None;
        }

        unsafe {
            let spot_ref = &*self.raw.as_ptr();
            Some(SpotConfig {
                q: spot_ref.q,
                low_tail: spot_ref.low != 0,
                discard_anomalies: spot_ref.discard_anomalies != 0,
                level: spot_ref.level,
                max_excess: spot_ref.tail.peaks.container.capacity,
            })
        }
    }

    /// Get the total number of data points seen
    pub fn n(&self) -> usize {
        if !self.initialized {
            return 0;
        }

        unsafe {
            let spot_ref = &*self.raw.as_ptr();
            spot_ref.n as usize
        }
    }

    /// Get the total number of excesses
    pub fn nt(&self) -> usize {
        if !self.initialized {
            return 0;
        }

        unsafe {
            let spot_ref = &*self.raw.as_ptr();
            spot_ref.nt as usize
        }
    }

    /// Get the current tail parameters
    pub fn tail_parameters(&self) -> (f64, f64) {
        if !self.initialized {
            return (f64::NAN, f64::NAN);
        }

        unsafe {
            let spot_ref = &*self.raw.as_ptr();
            (spot_ref.tail.gamma, spot_ref.tail.sigma)
        }
    }
}

impl Drop for SpotDetector {
    fn drop(&mut self) {
        if self.initialized {
            unsafe {
                ffi::spot_free(self.raw.as_mut_ptr());
            }
        }
    }
}

// Safety: SpotDetector can be safely sent between threads as long as it's not used concurrently
unsafe impl Send for SpotDetector {}
