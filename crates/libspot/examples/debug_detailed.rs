//! Debug test to verify update_stats calls match between implementations

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

    // Run up to the divergence point and stop there
    let mut normal = 0;
    let mut excess = 0; 
    let mut anomaly = 0;

    for i in 0..500000 {
        let val = rng.rexp();
        match detector.step(val)? {
            SpotStatus::Normal => normal += 1,
            SpotStatus::Excess => excess += 1,
            SpotStatus::Anomaly => anomaly += 1,
        }
        
        // Log key statistics at specific intervals
        if i % 100000 == 99999 {
            println!("Step {}: ANOMALY={} EXCESS={} NORMAL={} Z={:.6} T={:.6}", 
                     i+1, anomaly, excess, normal,
                     detector.anomaly_threshold(),
                     detector.excess_threshold());
        }
    }

    println!("Final: ANOMALY={} EXCESS={} NORMAL={} Z={:.6} T={:.6}", 
             anomaly, excess, normal,
             detector.anomaly_threshold(),
             detector.excess_threshold());

    Ok(())
}