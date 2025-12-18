#![doc = include_str!("../README.md")]
//!
//! # Feature Flags
//!
//! - **`serde`** (enabled by default): Enables serialization and deserialization support for all model types
//!   using the [`serde`](https://serde.rs/) framework. This is useful for:
//!   - Saving trained models to disk
//!   - Loading pre-trained models for deployment
//!   - Sharing models between applications
//!   - Checkpointing during long-running processes
//!
//!   To disable serialization support (e.g., for minimal dependencies), use:
//!   ```toml
//!   [dependencies]
//!   libspot-rs = { version = "0.1", default-features = false }
//!   ```
//!
//! ## Example with Serialization
//!
//! ```toml
//! [dependencies]
//! libspot-rs = { version = "0.1" }  # serde is enabled by default
//! serde_json = "1.0"
//! ```
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
//! // Serialize to JSON (serde is enabled by default)
//! let json = serde_json::to_string(&spot).unwrap();
//!
//! // Save to file
//! std::fs::write("model.json", &json).unwrap();
//!
//! // Later, load and continue using
//! let json = std::fs::read_to_string("model.json").unwrap();
//! let mut loaded: SpotDetector = serde_json::from_str(&json).unwrap();
//! let status = loaded.step(50.0);
//! ```

mod config;
mod error;
mod estimator;
mod math;
mod p2;
mod peaks;
#[cfg(feature = "serde")]
mod ser;
mod spot;
mod status;
mod tail;
mod ubend;

// Re-export public types
pub use config::SpotConfig;
pub use error::{SpotError, SpotResult};
pub use peaks::Peaks;
pub use spot::SpotDetector;
pub use status::SpotStatus;
pub use tail::Tail;
pub use ubend::Ubend;

// Re-export commonly used types to match libspot crate
pub use f64 as SpotFloat;

/// Get the version of the pure Rust libspot implementation
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}
