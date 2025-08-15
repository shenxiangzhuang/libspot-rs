//! Detailed GPD parameter tracking to find exact divergence point
//!
//! This tracks gamma and sigma evolution step by step to understand
//! exactly when and why the parameters diverge from the C implementation.

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
    println!("Detailed GPD parameter evolution tracking");

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

    // Generate initial training data
    let n = 20000;
    let mut initial_data = Vec::with_capacity(n);
    let mut rng = CRand::new(1);

    for _ in 0..n {
        initial_data.push(rng.rexp());
    }

    // Fit the model
    detector.fit(&initial_data)?;
    let (initial_gamma, initial_sigma) = detector.tail_parameters();
    println!("After initial fit: gamma={:.9}, sigma={:.9}", initial_gamma, initial_sigma);
    println!("T: {:.9}, Z: {:.9}", detector.excess_threshold(), detector.anomaly_threshold());

    // Track parameter evolution during detection
    let mut normal = 0;
    let mut excess = 0;
    let mut anomaly = 0;
    let mut excess_count = 0;
    
    // Track changes at critical points
    let track_points = [1000, 5000, 10000, 20000, 50000, 100000, 200000, 500000];
    let mut next_track_idx = 0;

    for i in 0..1_000_000 {
        let val = rng.rexp();
        let prev_gamma = detector.tail_parameters().0;
        let prev_sigma = detector.tail_parameters().1;
        
        match detector.step(val)? {
            SpotStatus::Normal => normal += 1,
            SpotStatus::Excess => {
                excess += 1;
                excess_count += 1;
                
                // Check if parameters changed significantly
                let (new_gamma, new_sigma) = detector.tail_parameters();
                let gamma_change = (new_gamma - prev_gamma).abs();
                let sigma_change = (new_sigma - prev_sigma).abs();
                
                if gamma_change > 0.01 || sigma_change > 0.01 {
                    println!("Step {}: Large param change - gamma: {:.9} -> {:.9} (Δ={:.9}), sigma: {:.9} -> {:.9} (Δ={:.9})", 
                             i + 1, prev_gamma, new_gamma, gamma_change, prev_sigma, new_sigma, sigma_change);
                }
                
                // Track every 10th excess for the first 100 excesses
                if excess_count <= 100 && excess_count % 10 == 0 {
                    println!("Excess #{}: gamma={:.9}, sigma={:.9}, Z={:.9}", 
                             excess_count, new_gamma, new_sigma, detector.anomaly_threshold());
                }
            },
            SpotStatus::Anomaly => anomaly += 1,
        }
        
        // Check tracking points
        if next_track_idx < track_points.len() && i + 1 == track_points[next_track_idx] {
            let (gamma, sigma) = detector.tail_parameters();
            println!("\nTrack point {}: ANOMALY={} EXCESS={} NORMAL={}", 
                     track_points[next_track_idx], anomaly, excess, normal);
            println!("  gamma={:.9}, sigma={:.9}, Z={:.9}, T={:.9}", 
                     gamma, sigma, detector.anomaly_threshold(), detector.excess_threshold());
            println!("  n={}, nt={}, s={:.9}", 
                     detector.n(), detector.nt(), detector.nt() as f64 / detector.n() as f64);
            next_track_idx += 1;
        }
    }

    println!("\nFinal results:");
    println!("ANOMALY={} EXCESS={} NORMAL={}", anomaly, excess, normal);
    let (final_gamma, final_sigma) = detector.tail_parameters();
    println!("Final GPD: gamma={:.9}, sigma={:.9}", final_gamma, final_sigma);
    println!("Z={:.9} T={:.9}", detector.anomaly_threshold(), detector.excess_threshold());

    Ok(())
}