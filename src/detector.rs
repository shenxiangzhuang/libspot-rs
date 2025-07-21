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
        // Initialize allocators (conditionally compiled for target)
        ffi::setup_allocators();

        let mut detector = SpotDetector {
            raw: MaybeUninit::uninit(),
            initialized: false,
        };

        // Call the appropriate FFI function based on target
        #[cfg(not(target_arch = "wasm32"))]
        let status = unsafe {
            ffi::spot_init(
                detector.raw.as_mut_ptr(),
                config.q,
                if config.low_tail { 1 } else { 0 },
                if config.discard_anomalies { 1 } else { 0 },
                config.level,
                config.max_excess,
            )
        };
        
        #[cfg(target_arch = "wasm32")]
        let status = ffi::spot_init(
            detector.raw.as_mut_ptr(),
            config.q,
            if config.low_tail { 1 } else { 0 },
            if config.discard_anomalies { 1 } else { 0 },
            config.level,
            config.max_excess as u32, // Convert to u32 for WASM
        );

        if status < 0 {
            return Err(SpotError::from_code(status));
        }

        detector.initialized = true;
        Ok(detector)
    }

    /// Fit the model using initial training data
    pub fn fit(&mut self, data: &[f64]) -> SpotResult<()> {
        if !self.initialized {
            return Err(SpotError::NotInitialized);
        }

        // Use conditional compilation to handle the safety requirements differently
        #[cfg(not(target_arch = "wasm32"))]
        let status = unsafe {
            ffi::spot_fit(self.raw.as_mut_ptr(), data.as_ptr(), data.len() as c_ulong)
        };
        
        #[cfg(target_arch = "wasm32")]
        let status = ffi::spot_fit(self.raw.as_mut_ptr(), data.as_ptr(), data.len() as c_ulong);

        if status < 0 {
            return Err(SpotError::from_code(status));
        }

        Ok(())
    }

    /// Process a single data point and return its classification
    pub fn step(&mut self, value: f64) -> SpotResult<SpotStatus> {
        if !self.initialized {
            return Err(SpotError::NotInitialized);
        }

        // Use conditional compilation to handle the safety requirements differently
        #[cfg(not(target_arch = "wasm32"))]
        let status = unsafe {
            ffi::spot_step(self.raw.as_mut_ptr(), value)
        };
        
        #[cfg(target_arch = "wasm32")]
        let status = ffi::spot_step(self.raw.as_mut_ptr(), value);
        
        if status < 0 {
            return Err(SpotError::from_code(status));
        }
        Ok(SpotStatus::from(status))
    }

    /// Get the quantile for a given probability
    pub fn quantile(&self, q: f64) -> f64 {
        if !self.initialized {
            return f64::NAN;
        }

        // Use conditional compilation to handle the safety requirements differently
        #[cfg(not(target_arch = "wasm32"))]
        let result = unsafe {
            ffi::spot_quantile(self.raw.as_ptr(), q)
        };
        
        #[cfg(target_arch = "wasm32")]
        let result = ffi::spot_quantile(self.raw.as_ptr(), q);
        
        result
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
                max_excess: spot_ref.tail.peaks.container.capacity as u64,
            })
        }
    }
}

impl Drop for SpotDetector {
    fn drop(&mut self) {
        if self.initialized {
            // Use conditional compilation to handle the safety requirements differently
            #[cfg(not(target_arch = "wasm32"))]
            unsafe {
                ffi::spot_free(self.raw.as_mut_ptr());
            }
            
            #[cfg(target_arch = "wasm32")]
            ffi::spot_free(self.raw.as_mut_ptr());
        }
    }
}

// Safety: SpotDetector can be safely sent between threads as long as it's not used concurrently
unsafe impl Send for SpotDetector {}

// WASM-specific implementation
#[cfg(target_arch = "wasm32")]
pub mod wasm {
    use wasm_bindgen::prelude::*;
    use super::*;
    use js_sys::Float64Array;
    
    /// WASM-compatible wrapper for SpotDetector
    #[wasm_bindgen]
    pub struct WasmSpotDetector {
        detector: SpotDetector,
    }
    
    #[wasm_bindgen]
    impl WasmSpotDetector {
        /// Create a new SPOT detector with the given configuration
        #[wasm_bindgen(constructor)]
        pub fn new(q: f64, low_tail: bool, discard_anomalies: bool, level: f64, max_excess: u32) -> Result<WasmSpotDetector, JsValue> {
            // Create config from JS parameters
            let config = SpotConfig {
                q,
                low_tail,
                discard_anomalies,
                level,
                max_excess: max_excess as u64,
            };
            
            // Create detector
            match SpotDetector::new(config) {
                Ok(detector) => Ok(WasmSpotDetector { detector }),
                Err(e) => Err(JsValue::from_str(&format!("Failed to create detector: {:?}", e))),
            }
        }
        
        /// Fit the model using initial training data
        #[wasm_bindgen]
        pub fn fit(&mut self, data: &Float64Array) -> Result<(), JsValue> {
            // Convert JS array to Rust slice
            let data_vec = data.to_vec();
            
            // Call the Rust implementation
            match self.detector.fit(&data_vec) {
                Ok(_) => Ok(()),
                Err(e) => Err(JsValue::from_str(&format!("Failed to fit model: {:?}", e))),
            }
        }
        
        /// Process a single data point and return its classification
        #[wasm_bindgen]
        pub fn step(&mut self, value: f64) -> Result<i32, JsValue> {
            // Call the Rust implementation
            match self.detector.step(value) {
                Ok(status) => Ok(status as i32),
                Err(e) => Err(JsValue::from_str(&format!("Failed to process data point: {:?}", e))),
            }
        }
        
        /// Get the quantile for a given probability
        #[wasm_bindgen]
        pub fn quantile(&self, q: f64) -> f64 {
            self.detector.quantile(q)
        }
        
        /// Get the current anomaly threshold
        #[wasm_bindgen]
        pub fn anomaly_threshold(&self) -> f64 {
            self.detector.anomaly_threshold()
        }
        
        /// Get the current excess threshold
        #[wasm_bindgen]
        pub fn excess_threshold(&self) -> f64 {
            self.detector.excess_threshold()
        }
    }
}