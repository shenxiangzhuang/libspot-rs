//! Short debug test to see Grimshaw behavior

use libspot::{Spot, SpotConfig, SpotStatus};

/// Random number generator that matches C's rand()/srand() for reproducible results
pub struct CRand;

impl CRand {
    pub fn new(seed: u32) -> Self {
        unsafe {
            libc::srand(seed);
        }
        CRand
    }

    pub fn rand(&mut self) -> u32 {
        unsafe { libc::rand() as u32 }
    }

    pub fn runif(&mut self) -> f64 {
        self.rand() as f64 / 2147483647.0 
    }

    pub fn rexp(&mut self) -> f64 {
        -self.runif().ln()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SpotConfig {
        q: 0.0001,
        low_tail: false,
        discard_anomalies: true,
        level: 0.998,
        max_excess: 200,
    };

    let mut detector = Spot::new(config)?;
    let mut rng = CRand::new(1);

    // Training data
    let n = 20000;
    let mut initial_data = Vec::with_capacity(n);
    for _ in 0..n {
        initial_data.push(rng.rexp());
    }
    detector.fit(&initial_data)?;

    // Run a very small test to see Grimshaw behavior
    for i in 0..200 {
        let val = rng.rexp();
        let status = detector.step(val)?;
        
        // Log whenever we see an excess that triggers tail fitting
        if matches!(status, SpotStatus::Excess | SpotStatus::Anomaly) {
            println!("Step {}: Status={:?}, Z={:.6}, T={:.6}", 
                     i, status, 
                     detector.anomaly_threshold(),
                     detector.excess_threshold());
        }
    }

    Ok(())
}