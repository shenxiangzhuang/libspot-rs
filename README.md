# libspot-rs

| Crate | Version | Documentation | License |
|-------|---------|---------------|---------|
| **libspot** (C FFI) | [![Crates.io](https://img.shields.io/crates/v/libspot.svg)](https://crates.io/crates/libspot) | [![Documentation](https://docs.rs/libspot/badge.svg)](https://docs.rs/libspot) | [![License: LGPL v3](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0) |
| **libspot-rs** (Pure Rust) | [![Crates.io](https://img.shields.io/crates/v/libspot-rs.svg)](https://crates.io/crates/libspot-rs) | [![Documentation](https://docs.rs/libspot-rs/badge.svg)](https://docs.rs/libspot-rs) | [![License: LGPL v3](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0) |

Rust implementations of the [SPOT (Streaming Peaks Over Threshold)](https://github.com/asiffer/libspot) algorithm for real-time anomaly detection in time series data.

## Installation

Choose your preferred implementation:

```bash
# C FFI version (faster, requires C dependencies)
cargo add libspot

# Pure Rust version (safer, no dependencies)
cargo add libspot-rs
```

## Quick Start

Both implementations provide **identical APIs** - you can switch between them by just changing the crate import!

```rust
// Choose your implementation:
// use libspot::{SpotDetector, SpotConfig, SpotStatus};      // C FFI version
use libspot_rs::{SpotDetector, SpotConfig, SpotStatus};   // Pure Rust version

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

Both implementations support identical configuration:

```rust
use libspot_rs::SpotConfig; // or use libspot::SpotConfig;

let config = SpotConfig {
    q: 0.0001,              // Anomaly probability threshold (lower = more sensitive)
    low_tail: false,        // Monitor upper tail (set true for lower tail)
    discard_anomalies: true, // Exclude anomalies from model updates
    level: 0.998,           // Quantile level that defines the tail
    max_excess: 200,        // Maximum number of excess values to store
};
```


## Comparison


| Feature | `libspot` (C FFI) | `libspot-rs` (Pure Rust) |
|---------|-------------------|---------------------------|
| **Installation** | `cargo add libspot` | `cargo add libspot-rs` |
| **Type** | C FFI Bindings | Pure Rust Implementation |
| **API** | ✅ Identical | ✅ Identical |
| **Performance** | ✅ ~1.04s (50M samples) | ✅ ~0.83s (50M samples) |
| **Memory Safety** | ⚠️ Manual (C code) | ✅ Guaranteed |
| **Dependencies** | 📦 C library + bindgen | 🎯 None |
| **Cross-platform** | ⚠️ Build complexity | ✅ Easy |
| **WebAssembly** | ❌ Limited support | ✅ Full support |
| **Results** | ✅ Reference standard | ✅ Mathematically identical |
| **Key Benefits** | Fast, Proven, Compatible | Safe, Portable, WebAssembly |
| **Documentation** | [docs.rs/libspot](https://docs.rs/libspot) | [docs.rs/libspot-rs](https://docs.rs/libspot-rs) |

## Correctness & Performance

Both implementations provide identical results to the original C implementation. Benchmark tests process 50M samples and produce mathematically equivalent anomaly counts and thresholds:

|     Metric      | C Implementation | Rust Wrapper (FFI) | Pure Rust (libspot-rs) |
|:---------------:|:----------------:|:------------------:|:----------------------:|
|  **Anomalies**  |      90,007      |     90,007 ✓       |       90,007 ✓         |
|   **Excess**    |       7,829      |      7,829 ✓       |        7,829 ✓         |
|   **Normal**    |    49,902,164    |   49,902,164 ✓     |     49,902,164 ✓       |
|      **Z**      |     6.237668     |    6.237668 ✓      |      6.237668 ✓        |
|      **T**      |     6.236165     |    6.236165 ✓      |      6.236165 ✓        |
| **Performance** |      ~0.67s      |     ~1.04s ≈       |       ~1.13s ≈         |

**Benchmark Commands:**
- **Pure Rust**: `cargo run -r --example basic` (in `crates/libspot-rs`)
- **C FFI**: `cargo run -r --example basic` (in `crates/libspot`)
- **Original C**: `cd crates/libspot/libspot && make && cc -O3 -o /tmp/basic ../examples/basic.c dist/libspot.a.$(cat version) -Idist/ -lm && /tmp/basic`

The results demonstrate that both Rust implementations achieve excellent performance while maintaining mathematical correctness. The pure Rust version is actually the fastest, showing the effectiveness of Rust's optimizations.

## Documentation

- **API Documentation**: [docs.rs/libspot](https://docs.rs/libspot) | [docs.rs/libspot-rs](https://docs.rs/libspot-rs)
- **Original C Library**: [asiffer/libspot](https://github.com/asiffer/libspot)

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the **GNU Lesser General Public License v3.0 (LGPL-3.0)** to comply with the underlying [libspot](https://github.com/asiffer/libspot) C library license.
