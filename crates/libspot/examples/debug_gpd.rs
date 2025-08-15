//! Debug GPD parameters to compare pure Rust vs C implementation
//!
//! This example extracts the actual gamma and sigma parameters
//! at key points to understand where the estimation diverges.

use libspot::{Spot, SpotConfig, SpotStatus};

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
    println!("Debug GPD parameters - Pure Rust implementation");

    // Configure SPOT detector
    let config = SpotConfig {
        q: 0.0001,               // anomaly probability
        low_tail: false,         // observe upper tail
        discard_anomalies: true, // flag anomalies
        level: 0.998,            // tail quantile
        max_excess: 200,         // data points to keep
    };

    // Create and initialize SPOT detector
    let mut detector = Spot::new(config.clone())?;

    // Generate initial training data
    let n = 20000;
    let mut initial_data = Vec::with_capacity(n);
    let mut rng = CRand::new(1);

    for _ in 0..n {
        initial_data.push(rng.rexp());
    }

    // Fit the model
    detector.fit(&initial_data)?;
    println!("After fitting with {} samples:", n);
    println!("  T: {:.9}", detector.excess_threshold());
    println!("  Z: {:.9}", detector.anomaly_threshold());
    
    // Try to extract GPD parameters - we need to add accessors for debugging
    // For now, let's track detection at key sample points

    let key_samples = [100000, 500000, 1000000];
    
    for &target in &key_samples {
        // Reset detector
        let mut detector = Spot::new(config.clone())?;
        detector.fit(&initial_data)?;
        let mut rng = CRand::new(1);
        
        // Skip initial data
        for _ in 0..n {
            rng.rexp();
        }

        let mut normal = 0;
        let mut excess = 0;
        let mut anomaly = 0;

        for i in 0..target {
            let val = rng.rexp();
            match detector.step(val)? {
                SpotStatus::Normal => normal += 1,
                SpotStatus::Excess => excess += 1,
                SpotStatus::Anomaly => anomaly += 1,
            }
        }

        println!("\nAfter {} detection samples:", target);
        println!("  ANOMALY={} EXCESS={} NORMAL={}", anomaly, excess, normal);
        println!("  T: {:.9}", detector.excess_threshold());
        println!("  Z: {:.9}", detector.anomaly_threshold());
        println!("  n={}, nt={}", detector.n(), detector.nt());
        println!("  s (nt/n): {:.9}", detector.nt() as f64 / detector.n() as f64);
        
        // Get GPD parameters
        let (gamma, sigma) = detector.tail_parameters();
        println!("  GPD gamma: {:.9}", gamma);
        println!("  GPD sigma: {:.9}", sigma);
        
        // Calculate what the quantile should be at this point
        let s = detector.nt() as f64 / detector.n() as f64;
        let q = 0.0001;
        println!("  Expected anomaly calc: T + tail.quantile({:.9}, {:.9})", s, q);
    }

    Ok(())
}