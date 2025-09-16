# libspot

[![Crates.io](https://img.shields.io/crates/v/libspot.svg)](https://crates.io/crates/libspot)
[![Documentation](https://docs.rs/libspot/badge.svg)](https://docs.rs/libspot)
[![License: LGPL v3](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)

A safe Rust wrapper (using FFI) for the [libspot](https://github.com/asiffer/libspot) time series anomaly detection library.

## Quick Start

```rust
use libspot::{SpotDetector, SpotConfig, SpotStatus};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create detector with default configuration
    let config = SpotConfig::default();
    let mut detector = SpotDetector::new(config)?;

    // Fit with training data
    let training_data: Vec<f64> = (0..1000)
        .map(|i| 5.0 + (i as f64 * 0.01).sin() * 2.0)
        .collect();
    detector.fit(&training_data)?;

    // Detect anomalies in real-time
    let test_value = 50.0; // This should be an anomaly
    match detector.step(test_value)? {
        SpotStatus::Normal => println!("Normal data point"),
        SpotStatus::Excess => println!("In the tail distribution"),
        SpotStatus::Anomaly => println!("Anomaly detected! 🚨"),
    }

    Ok(())
}
```


## Alternative

For a pure Rust implementation without FFI dependencies, see the [`libspot-rs`](https://crates.io/crates/libspot-rs) crate.

## License

This project is licensed under the **GNU Lesser General Public License v3.0 (LGPL-3.0)**.
