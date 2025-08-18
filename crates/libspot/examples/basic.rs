//! Basic example demonstrating the libspot library
//!
//! This example replicates the C libspot example but using the safe Rust API.
//! It performs a comprehensive benchmark with 50 million samples.

use libspot::{version, SpotConfig, SpotDetector, SpotStatus};
use std::time::Instant;

/// Random number generator that matches C's rand()/srand() for reproducible results
pub struct CRand;

impl CRand {
    /// Create a new random number generator with the given seed
    pub fn new(seed: u32) -> Self {
        unsafe {
            libc::srand(seed);
        }
        CRand
    }

    /// Generate a random integer
    pub fn rand(&mut self) -> u32 {
        unsafe { libc::rand() as u32 }
    }

    /// Generate a uniform random float in [0, 1)
    pub fn runif(&mut self) -> f64 {
        self.rand() as f64 / 2147483647.0 // RAND_MAX = 2^31 - 1
    }

    /// Generate an exponentially distributed random variable with rate 1
    pub fn rexp(&mut self) -> f64 {
        -self.runif().ln()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing libspot from Rust using the safe API!");

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
    let mut detector = SpotDetector::new(config)?;
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
