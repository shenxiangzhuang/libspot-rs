//! Targeted debug test to compare FFI Grimshaw estimator at divergence point

use libspot_ffi::{SpotConfig, SpotDetector, SpotStatus};
use std::fs::File;
use std::io::Write;

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

    let mut detector = SpotDetector::new(config)?;
    let mut rng = CRand::new(1);

    // Training data
    let n = 20000;
    let mut initial_data = Vec::with_capacity(n);
    for _ in 0..n {
        initial_data.push(rng.rexp());
    }
    detector.fit(&initial_data)?;

    // Run exactly to the divergence point and capture the state
    let mut anomaly_count = 0;
    let mut excess_count = 0;
    
    // Run exactly 99,999 steps (just before the known divergence)
    for _i in 0..99999 {
        let val = rng.rexp();
        match detector.step(val)? {
            SpotStatus::Normal => {},
            SpotStatus::Excess => excess_count += 1,
            SpotStatus::Anomaly => anomaly_count += 1,
        }
    }
    
    println!("After 99,999 steps: ANOMALY={} EXCESS={} Z={:.6} T={:.6}",
             anomaly_count, excess_count,
             detector.anomaly_threshold(),
             detector.excess_threshold());

    // Now process the 100,000th sample and capture detailed state
    let val_100k = rng.rexp();
    println!("100,000th sample value: {:.6}", val_100k);
    
    // Process the sample
    match detector.step(val_100k)? {
        SpotStatus::Normal => println!("Sample 100k: Normal"),
        SpotStatus::Excess => {
            excess_count += 1;
            println!("Sample 100k: Excess (new total: {})", excess_count);
        },
        SpotStatus::Anomaly => {
            anomaly_count += 1;
            println!("Sample 100k: Anomaly (new total: {})", anomaly_count);
        },
    }
    
    println!("After 100,000 steps: ANOMALY={} EXCESS={} Z={:.6} T={:.6}",
             anomaly_count, excess_count,
             detector.anomaly_threshold(),
             detector.excess_threshold());

    // Export the current state for manual comparison
    let mut file = File::create("/tmp/peaks_data_100k_ffi.txt")?;
    writeln!(file, "Current Z: {:.15}", detector.anomaly_threshold())?;
    writeln!(file, "Current T: {:.15}", detector.excess_threshold())?;
    
    println!("FFI data exported to /tmp/peaks_data_100k_ffi.txt");

    Ok(())
}