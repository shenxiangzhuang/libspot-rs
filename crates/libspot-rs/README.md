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
    let score = detector.anomaly_score(test_value);
    println!("Anomaly score: {score:.6}");

    match detector.step(test_value)? {
        SpotStatus::Normal => println!("Normal data point"),
        SpotStatus::Excess => println!("In the tail distribution"),
        SpotStatus::Anomaly => println!("Anomaly detected! 🚨"),
    }

    Ok(())
}
```

## Anomaly score

`SpotDetector::anomaly_score(value)` converts the fitted Generalized Pareto
tail into a bounded conditional percentile. Let

$$
y(x)=a(x-t),\qquad
a=\begin{cases}
1 & \text{upper tail},\\
-1 & \text{lower tail}.
\end{cases}
$$

Then the score is the GPD cumulative distribution:

$$
A(x)=\begin{cases}
0, & y\le0,\\
1-\left(1+\dfrac{\gamma y}{\sigma}\right)^{-1/\gamma},
    & y>0,\ \gamma\ne0,\\
1-\exp\left(-\dfrac{y}{\sigma}\right),
    & y>0,\ \gamma=0.
\end{cases}
$$

The score is `0` outside the modeled tail and approaches `1` for increasingly
extreme observations. It is a percentile within the selected tail, not the
posterior probability that the observation is anomalous. The method is
read-only: it does not change counters, peaks, fitted parameters, or thresholds.

At the current anomaly threshold `z_q`, the expected score is:

$$
A(z_q)=1-\frac{q}{s},\qquad s=\frac{N_t}{n}.
$$

For two-sided detection, keep the existing independent upper- and lower-tail
detectors and combine their read-only scores with `upper_score.max(lower_score)`.
Each detector continues to update its own state through `step` exactly as
before.

## Features

### Estimator and threshold options

`SpotDetector::new(config)` keeps the default behavior: it uses `SpotEstimator::Best`, the P2 initial threshold, and the historical `>=` excess update rule.

To reproduce algorithms that explicitly use FluxEV-style MOM-SPOT, configure all options explicitly:

```rust,ignore
use libspot_rs::{
    SpotConfig, SpotDetector, SpotEstimator, SpotExcessUpdate, SpotInitialThreshold,
};

let config = SpotConfig {
    q: 0.001,
    level: 0.98,
    max_excess: 10_000,
    ..SpotConfig::default()
};

let mut detector = SpotDetector::new_with_full_options(
    config,
    SpotEstimator::Mom,
    SpotInitialThreshold::Empirical,
    SpotExcessUpdate::Greater,
)?;
```

### Serialization (Model Persistence)

Serialization support is **enabled by default**. SPOT detectors can be serialized and deserialized for model deployment:

```toml
[dependencies]
libspot-rs = { version = "0.4.0-rc.1" }  # serde is enabled by default
serde_json = "1.0"
```

To disable serialization support (e.g., for minimal dependencies), use:
```toml
[dependencies]
libspot-rs = { version = "0.4.0-rc.1", default-features = false }
```

This enables:
- **Model persistence**: Save trained models to disk and load them later
- **Model deployment**: Export models for use in production systems
- **Model sharing**: Share trained models between applications
- **Checkpointing**: Save model state during long-running processes

Example usage:

```rust,ignore
use libspot_rs::{SpotConfig, SpotDetector};
use serde_json;

// Train a model
let config = SpotConfig::default();
let mut spot = SpotDetector::new(config).unwrap();
let training_data: Vec<f64> = (0..1000).map(|i| i as f64 / 100.0).collect();
spot.fit(&training_data).unwrap();

// Save the model to a JSON file
let json = serde_json::to_string_pretty(&spot).unwrap();
std::fs::write("model.json", &json).unwrap();

// Later, load the model and continue using it
let json = std::fs::read_to_string("model.json").unwrap();
let mut loaded: SpotDetector = serde_json::from_str(&json).unwrap();

// The loaded model is ready to use immediately
let status = loaded.step(50.0).unwrap();
```

The serialization handles special float values (NaN, Infinity) correctly, ensuring that models can be reliably saved and restored.

## Alternative

For C FFI bindings to the original libspot library, see the [`libspot`](https://crates.io/crates/libspot) crate.

## License

This project is licensed under the GNU Lesser General Public License v3.0 - see the [LICENSE](https://github.com/shenxiangzhuang/libspot-rs/blob/main/LICENSE) file for details.
}
