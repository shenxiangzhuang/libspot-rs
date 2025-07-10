# libspot-rs

A safe Rust wrapper for the [libspot](https://github.com/asiffer/libspot) time series anomaly detection library.

## Quick Start

```rust
use libspot::{SpotDetector, SpotConfig, SpotStatus};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create detector with default configuration
    let config = SpotConfig::default();
    let mut detector = SpotDetector::new(config)?;

    // Fit with training data (normal distribution around 5.0)
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

## Examples

See the [examples](./examples) directory for more usage examples.
You can run the examples with `cargo run --example <example_name>`:

```bash
cargo run --example simple
cargo run --example basic
```

## License

This project is licensed under the **GNU Lesser General Public License v3.0 (LGPL-3.0)**
to comply with the underlying [libspot](https://github.com/asiffer/libspot) C library license.
