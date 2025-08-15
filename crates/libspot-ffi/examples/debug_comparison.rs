//! Debug comparison using FFI to match C implementation exactly
//!
//! This example runs the FFI implementation and shows intermediate values
//! to compare against the pure Rust implementation.

use libspot_ffi::{version, SpotConfig, SpotDetector, SpotStatus};

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
    println!("Debug comparison using FFI (C implementation)");

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
    let mut detector = SpotDetector::new(config.clone())?;
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
    println!("After fitting - T: {:.6}", detector.excess_threshold());

    // Test 1M samples to compare with pure Rust
    let k = 1_000_000;
    let mut normal = 0;
    let mut excess = 0;
    let mut anomaly = 0;

    for i in 0..k {
        let val = rng.rexp();
        match detector.step(val)? {
            SpotStatus::Normal => normal += 1,
            SpotStatus::Excess => excess += 1,
            SpotStatus::Anomaly => anomaly += 1,
        }
        
        // Print intermediate results every 10k steps at key points
        if (i + 1) % 10000 == 0 && ((i + 1) >= 920000 || (i + 1) % 100000 == 0) {
            println!(
                "FFI Step {}: ANOMALY={} EXCESS={} NORMAL={} Z={:.6} T={:.6}",
                i + 1, anomaly, excess, normal,
                detector.anomaly_threshold(),
                detector.excess_threshold()
            );
        }
    }

    println!("\n=== FFI RESULTS FOR {} SAMPLES ===", k);
    println!("ANOMALY={} EXCESS={} NORMAL={}", anomaly, excess, normal);
    println!("Z={:.6} T={:.6}", detector.anomaly_threshold(), detector.excess_threshold());
    println!();

    Ok(())
}