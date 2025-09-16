# libspot-rs

[![Crates.io](https://img.shields.io/crates/v/libspot-rs.svg)](https://crates.io/crates/libspot-rs)
[![Documentation](https://docs.rs/libspot-rs/badge.svg)](https://docs.rs/libspot-rs)
[![License: LGPL v3](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)

A pure Rust implementation of the [SPOT (Streaming Peaks Over Threshold)](https://github.com/asiffer/libspot) algorithm for real-time anomaly detection in time series data.

## Quick Start

```rust
use libspot_rs::{SpotDetector, SpotConfig, SpotStatus};

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

For C FFI bindings to the original libspot library, see the [`libspot`](https://crates.io/crates/libspot) crate.

## License

This project is licensed under the GNU Lesser General Public License v3.0 - see the [LICENSE](https://github.com/shenxiangzhuang/libspot-rs/blob/main/LICENSE) file for details.
}