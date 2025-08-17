//! Pure Rust implementation of the SPOT algorithm for time series anomaly detection
//!
//! This crate provides a pure Rust implementation of the SPOT (Streaming Peaks Over Threshold)
//! algorithm, which is used for anomaly detection in time series data.
//!
//! # Key Components
//!
//! - [`Ubend`]: Circular buffer for storing data
//! - [`Peaks`]: Statistics computation over peaks data  
//! - [`Tail`]: Generalized Pareto Distribution tail modeling
//! - [`Spot`]: Main SPOT detector implementation
//!
//! # Usage
//!
//! ```rust
//! use spot_rs::{Spot, SpotConfig};
//!
//! let config = SpotConfig {
//!     q: 0.0001,
//!     low_tail: false,
//!     discard_anomalies: true,
//!     level: 0.998,
//!     max_excess: 200,
//! };
//! 
//! let mut detector = Spot::new(config).unwrap();
//! // ... use detector
//! ```

use std::os::raw::c_double;

/// Type alias for floating-point values to ensure C compatibility
/// Using c_double throughout eliminates precision differences with C implementation
pub type Float = c_double;

mod config;
mod error;
mod estimator;
pub mod math;
mod p2;
mod peaks;
mod spot;
mod status;
mod tail;
mod ubend;

// Re-export public types
pub use config::SpotConfig;
pub use error::{SpotError, SpotResult};
pub use spot::Spot;
pub use status::SpotStatus;
pub use ubend::Ubend;
pub use peaks::Peaks;
pub use tail::Tail;
pub use p2::p2_quantile;
