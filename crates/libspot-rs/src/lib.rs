#![doc = include_str!("../../../README.md")]

mod config;
mod error;
mod estimator;
mod math;
mod p2;
mod peaks;
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
