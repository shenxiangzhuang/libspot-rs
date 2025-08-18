# libspot-rs

[![Crates.io](https://img.shields.io/crates/v/libspot-rs.svg)](https://crates.io/crates/libspot-rs)
[![Documentation](https://docs.rs/libspot-rs/badge.svg)](https://docs.rs/libspot-rs)
[![License: LGPL v3](https://img.shields.io/badge/License-LGPL%20v3-blue.svg)](https://www.gnu.org/licenses/lgpl-3.0)

A pure Rust implementation of the [SPOT (Streaming Peaks Over Threshold)](https://github.com/asiffer/libspot) algorithm for real-time anomaly detection in time series data.

## Features

- **Pure Rust**: No external C dependencies required
- **Real-time**: Designed for streaming data processing
- **Configurable**: Flexible parameters for different use cases
- **Efficient**: Optimized for performance with minimal memory footprint
- **Well-documented**: Comprehensive API documentation and examples

## Installation

```bash
cargo add libspot-rs
```

## Quick Start

```rust
use libspot_rs::{Spot, SpotConfig, SpotStatus};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// Create detector with default configuration
let config = SpotConfig::default();
let mut detector = Spot::new(config)?;

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
# Ok(())
# }
```

## Configuration

The SPOT algorithm can be configured with various parameters:

```rust
use libspot_rs::SpotConfig;

let config = SpotConfig {
    q: 0.0001,              // Anomaly probability threshold
    low_tail: false,        // Monitor upper tail (set true for lower tail)
    discard_anomalies: true, // Exclude anomalies from model updates
    level: 0.998,           // Quantile level that defines the tail
    max_excess: 200,        // Maximum number of excess values to store
};
```

## Advanced Usage

### Custom Configuration

```rust
use libspot_rs::{Spot, SpotConfig};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
// More sensitive detector (lower anomaly threshold)
let sensitive_config = SpotConfig {
    q: 0.01,               // Higher probability = more sensitive
    level: 0.95,           // Lower level = larger tail region
    ..SpotConfig::default()
};

let mut detector = Spot::new(sensitive_config)?;
# Ok(())
# }
```

### Monitoring Multiple Metrics

```rust
use libspot_rs::{Spot, SpotConfig};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let cpu_history = vec![1.0, 2.0, 3.0];
# let memory_history = vec![1.0, 2.0, 3.0];
# let network_history = vec![1.0, 2.0, 3.0];
# fn get_cpu_usage() -> f64 { 1.0 }
# fn get_memory_usage() -> f64 { 1.0 }
# fn get_network_usage() -> f64 { 1.0 }
// Create separate detectors for different metrics
let mut cpu_detector = Spot::new(SpotConfig::default())?;
let mut memory_detector = Spot::new(SpotConfig::default())?;
let mut network_detector = Spot::new(SpotConfig::default())?;

// Train each detector with historical data
cpu_detector.fit(&cpu_history)?;
memory_detector.fit(&memory_history)?;
network_detector.fit(&network_history)?;

// Monitor in real-time
for _ in 0..3 { // Limited loop for doctest
    let cpu_status = cpu_detector.step(get_cpu_usage())?;
    let memory_status = memory_detector.step(get_memory_usage())?;
    let network_status = network_detector.step(get_network_usage())?;

    // Handle anomalies...
    break; // Exit early for doctest
}
# Ok(())
# }
```

### Accessing Detector State

```rust
# use libspot_rs::{Spot, SpotConfig};
# fn main() -> Result<(), Box<dyn std::error::Error>> {
# let mut detector = Spot::new(SpotConfig::default())?;
# let training_data = vec![1.0, 2.0, 3.0];
# detector.fit(&training_data)?;
// Get detector statistics
println!("Total samples: {}", detector.n());
println!("Excess count: {}", detector.nt());
println!("Anomaly threshold: {}", detector.anomaly_threshold());
println!("Excess threshold: {}", detector.excess_threshold());

// Get tail distribution parameters
let (gamma, sigma) = detector.tail_parameters();
println!("Tail shape: {}, scale: {}", gamma, sigma);

// Get peaks statistics
println!("Peaks mean: {}", detector.peaks_mean());
println!("Peaks variance: {}", detector.peaks_variance());
# Ok(())
# }
```

## Algorithm Overview

The SPOT algorithm is designed for online anomaly detection in time series data using:

1. **Extreme Value Theory (EVT)**: Models the tail of the data distribution
2. **Generalized Pareto Distribution (GPD)**: Fits the distribution of excesses
3. **Dynamic Thresholding**: Adapts to changing data patterns
4. **Streaming Processing**: Processes one data point at a time

Key concepts:
- **Excess**: Values above a high quantile threshold
- **Tail**: The extreme region of the data distribution
- **Anomaly**: Values with probability below the configured threshold

## Key Components

- [`Spot`]: Main SPOT detector implementation
- [`SpotConfig`]: Configuration parameters for the detector
- [`SpotStatus`]: Status returned by the detector for each data point
- [`Ubend`]: Circular buffer for storing data
- [`Peaks`]: Statistics computation over peaks data
- [`Tail`]: Generalized Pareto Distribution tail modeling

## Performance

libspot-rs is optimized for real-time processing:

- **Memory**: O(max_excess) space complexity
- **Time**: O(1) amortized time per data point
- **Throughput**: Can process millions of data points per second

## Comparison with C Implementation

| Feature | libspot-rs (Pure Rust) | libspot (C + FFI) |
|---------|----------------------|-------------------|
| Dependencies | None | C library, bindgen |
| Memory Safety | ✅ Guaranteed | ⚠️ Manual management |
| Performance | ✅ Excellent | ✅ Excellent |
| Cross-platform | ✅ Easy | ⚠️ Build complexity |
| WebAssembly | ✅ Full support | ❌ Limited |

## Examples

See the [`examples/`](examples/) directory for more comprehensive usage examples:

- [`basic.rs`](examples/basic.rs): Basic usage with synthetic data

## Contributing

Contributions are welcome! Please feel free to submit issues and pull requests.

## License

This project is licensed under the GNU Lesser General Public License v3.0 - see the [LICENSE](https://github.com/shenxiangzhuang/libspot-rs/blob/main/LICENSE) file for details.
