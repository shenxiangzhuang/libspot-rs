use libspot::{Spot, SpotConfig, SpotStatus};
use std::env;

/// Random number generator that matches C's rand()/srand()
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
    env::set_var("SPOT_DEBUG_GRIMSHAW", "1");
    
    let config = SpotConfig {
        q: 0.0001,
        low_tail: false,
        discard_anomalies: true,
        level: 0.998,
        max_excess: 200,
    };

    let mut detector = Spot::new(config)?;
    let mut rng = CRand::new(1);

    // Fit model
    let n = 20000;
    let mut initial_data = Vec::with_capacity(n);
    for _ in 0..n {
        initial_data.push(rng.rexp());
    }
    detector.fit(&initial_data)?;

    // Test at intervals to find divergence point
    let intervals = [10000, 50000, 100000, 200000, 500000, 1000000];
    
    for &samples in &intervals {
        let mut normal = 0;
        let mut excess = 0;
        let mut anomaly = 0;

        for _ in 0..samples {
            let val = rng.rexp();
            match detector.step(val)? {
                SpotStatus::Normal => normal += 1,
                SpotStatus::Excess => excess += 1,
                SpotStatus::Anomaly => anomaly += 1,
            }
        }

        println!("At {} samples: ANOMALY={} EXCESS={} NORMAL={} Z={:.6} T={:.6}",
                samples, anomaly, excess, normal,
                detector.anomaly_threshold(),
                detector.excess_threshold());
    }

    Ok(())
}