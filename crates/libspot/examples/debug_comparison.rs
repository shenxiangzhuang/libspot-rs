//! Debug comparison between pure Rust and C implementations
//!
//! This example runs both implementations side-by-side and compares
//! intermediate values to find where the divergence occurs.

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
    println!("Debug comparison between pure Rust and expected C results");

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

    // Test a smaller number first to see where divergence starts
    let test_iterations = [1000, 10000, 100000, 1000000];
    
    for &k in &test_iterations {
        // Reset to same state
        let mut detector = Spot::new(config.clone())?;
        detector.fit(&initial_data)?;
        let mut rng = CRand::new(1);
        
        // Skip the initial training data generation
        for _ in 0..n {
            rng.rexp();
        }

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
            
            // Print intermediate results every 10k steps
            if (i + 1) % 10000 == 0 {
                println!(
                    "Step {}: ANOMALY={} EXCESS={} NORMAL={} Z={:.6} T={:.6}",
                    i + 1, anomaly, excess, normal,
                    detector.anomaly_threshold(),
                    detector.excess_threshold()
                );
            }
        }

        println!("\n=== RESULTS FOR {} SAMPLES ===", k);
        println!("ANOMALY={} EXCESS={} NORMAL={}", anomaly, excess, normal);
        println!("Z={:.6} T={:.6}", detector.anomaly_threshold(), detector.excess_threshold());
        
        // Expected results from C implementation (50M sample reference):
        // ANOMALY=25898 EXCESS=71938 NORMAL=49902164 Z=7.422655 T=6.236165
        
        // Check if thresholds match expected at 50M
        if k >= 1000000 {
            let expected_t = 6.236165;
            let expected_z = 7.422655; // This is what we expect at 50M
            
            println!("Expected T: {:.6}, Got T: {:.6}, Diff: {:.9}", 
                     expected_t, detector.excess_threshold(), 
                     (detector.excess_threshold() - expected_t).abs());
            println!("Expected Z: {:.6}, Got Z: {:.6}, Diff: {:.9}", 
                     expected_z, detector.anomaly_threshold(), 
                     (detector.anomaly_threshold() - expected_z).abs());
        }
        
        println!();
    }

    Ok(())
}