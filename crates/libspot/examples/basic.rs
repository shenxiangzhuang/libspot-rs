//! Basic example demonstrating the libspot library
//!
//! This example replicates the C libspot example but using the pure Rust API.
//! It performs a comprehensive benchmark with 50 million samples.

use libspot::{Spot, SpotConfig, SpotStatus};
use std::time::Instant;

/// Random number generator that matches C's rand()/srand() for reproducible results
pub struct CRand {
    state: u32,
}

impl CRand {
    /// Create a new random number generator with the given seed
    pub fn new(seed: u32) -> Self {
        CRand { state: seed }
    }

    /// Generate a random integer using linear congruential generator (LCG)
    /// This matches the behavior of C's rand()/srand() for reproducible results
    pub fn rand(&mut self) -> u32 {
        // LCG parameters used by many C standard libraries
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        (self.state / 65536) % 32768 // Return value in range [0, 32767]
    }

    /// Generate a uniform random float in [0, 1)
    pub fn runif(&mut self) -> f64 {
        self.rand() as f64 / 32768.0 // Make sure we don't get exactly 0 or 1
    }

    /// Generate an exponentially distributed random variable with rate 1
    pub fn rexp(&mut self) -> f64 {
        let u = self.runif();
        // Ensure u is never 0 to avoid ln(0) = -inf
        let safe_u = if u <= f64::EPSILON { f64::EPSILON } else { u };
        -safe_u.ln()
    }
}

/// Get pure Rust implementation version
fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing libspot from Rust using the pure Rust API!");

    // Get library version
    let lib_version = version();
    println!("libspot version: {lib_version}");

    // Configure SPOT detector
    let config = SpotConfig {
        q: 0.0001,               // anomaly probability
        low_tail: false,         // observe upper tail
        discard_anomalies: true, // flag anomalies
        level: 0.998,            // tail quantile
        max_excess: 200,         // data points to keep
    };

    // Create and initialize SPOT detector
    let mut detector = Spot::new(config)?;
    println!("SPOT detector created successfully");

    // Generate initial training data
    let n = 20000;
    let mut initial_data = Vec::with_capacity(n);
    let mut rng = CRand::new(1); // Use same seed as C example

    for _ in 0..n {
        initial_data.push(rng.rexp());
    }

    // Fit the model
    detector.fit(&initial_data)?;
    println!("Model successfully fitted with {n} data points");

    // Main detection loop
    let k = 50_000_000;
    let mut normal = 0;
    let mut excess = 0;
    let mut anomaly = 0;

    println!("Starting anomaly detection on {k} samples...");
    let start = Instant::now();

    for _ in 0..k {
        let val = rng.rexp();
        match detector.step(val)? {
            SpotStatus::Normal => normal += 1,
            SpotStatus::Excess => excess += 1,
            SpotStatus::Anomaly => anomaly += 1,
        }
    }

    let duration = start.elapsed();

    // Print results
    println!("{:.6}", duration.as_secs_f64());
    println!("ANOMALY={anomaly} EXCESS={excess} NORMAL={normal}");
    println!(
        "Z={:.6} T={:.6}",
        detector.anomaly_threshold(),
        detector.excess_threshold()
    );

    println!("Detection completed successfully!");

    Ok(())
}