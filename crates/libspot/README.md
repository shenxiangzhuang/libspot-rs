# libspot

[![Crates.io](https://img.shields.io/crates/v/libspot.svg)](https://crates.io/crates/libspot)


A safe Rust wrapper(using FFI) for the [libspot](https://github.com/asiffer/libspot) time series anomaly detection library.

## Installation

```bash
cargo add libspot
```

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

## Correctness & Performance

This wrapper provides identical results to the original C implementation. The [`basic.rs`](./examples/basic.rs) example processes 50M samples and produces the **exact same** anomaly counts and thresholds as the reference [`basic.c`](https://asiffer.github.io/libspot/20_get_started/) implementation:

|     Metric      | C Implementation | Rust Wrapper | Identical |
|:---------------:|:----------------:|:------------:|:---------:|
|  **Anomalies**  |      90,007      |    90,007    |     ✓     |
|   **Excess**    |       7,829      |     7,829    |     ✓     |
|   **Normal**    |    49,902,164    |  49,902,164  |     ✓     |
|      **Z**      |     6.237668     |   6.237668   |     ✓     |
|      **T**      |     6.236165     |   6.236165   |     ✓     |
| **Performance** |    ~1.276820s    |  ~1.372725s  |     ✓     |

We run the benchmark with:
- Rust: `cargo run -r --example basic`(with `-r` flag to run the example in release mode).
- C: `cc -O3 -o /tmp/basic examples/basic.c -Idist/ -Ldist/ -l:libspot.so.2.0b5 -lm && LD_LIBRARY_PATH=dist /tmp/basic`(with `-O3` flag to compile the example in release mode).

As you can see, the performance is **very close**. You may get different results due to the different hardware and environment, but the results should be very similar.

## License

This project is licensed under the **GNU Lesser General Public License v3.0 (LGPL-3.0)**
to comply with the underlying [libspot](https://github.com/asiffer/libspot) C library license.


## Alternative

For a pure Rust implementation without FFI dependencies, see the [`libspot-rs`](../libspot-rs) crate.
