use libspot_ffi::{SpotConfig, SpotDetector, SpotStatus};
use std::time::Instant;

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
    println!("Debugging FFI SPOT Implementation");

    let config = SpotConfig {
        q: 0.0001,
        low_tail: false,
        discard_anomalies: true,
        level: 0.998,
        max_excess: 200,
    };

    let mut detector = SpotDetector::new(config)?;
    let mut rng = CRand::new(1);

    // Generate training data
    let n = 20000;
    let mut initial_data = Vec::with_capacity(n);
    for _ in 0..n {
        initial_data.push(rng.rexp());
    }

    detector.fit(&initial_data)?;
    println!("Initial thresholds after fit:");
    println!("  Excess threshold: {:.15}", detector.excess_threshold());
    println!("  Anomaly threshold: {:.15}", detector.anomaly_threshold());

    // Test with smaller number to debug differences
    let test_samples = 1000;
    let mut normal = 0;
    let mut excess = 0;
    let mut anomaly = 0;
    
    println!("\nProcessing {} test samples...", test_samples);
    
    for i in 0..test_samples {
        let val = rng.rexp();
        let status = detector.step(val)?;
        
        match status {
            SpotStatus::Normal => normal += 1,
            SpotStatus::Excess => {
                excess += 1;
                if i < 10 {
                    println!("Sample {}: EXCESS val={:.6} threshold={:.6}", i, val, detector.excess_threshold());
                }
            },
            SpotStatus::Anomaly => {
                anomaly += 1;
                if i < 10 {
                    println!("Sample {}: ANOMALY val={:.6} threshold={:.6}", i, val, detector.anomaly_threshold());
                }
            },
        }
        
        if i < 10 || i % 100 == 0 {
            println!("Sample {}: val={:.6} status={:?} Z={:.6} T={:.6}", 
                     i, val, status, detector.anomaly_threshold(), detector.excess_threshold());
        }
    }

    println!("\nResults after {} samples:", test_samples);
    println!("ANOMALY={} EXCESS={} NORMAL={}", anomaly, excess, normal);
    println!("Z={:.15} T={:.15}", detector.anomaly_threshold(), detector.excess_threshold());
    
    // Compare with larger test
    println!("\nRunning larger test (100k samples)...");
    let larger_test = 100_000;
    let mut normal_large = 0;
    let mut excess_large = 0;
    let mut anomaly_large = 0;
    
    for _ in 0..larger_test {
        let val = rng.rexp();
        match detector.step(val)? {
            SpotStatus::Normal => normal_large += 1,
            SpotStatus::Excess => excess_large += 1,
            SpotStatus::Anomaly => anomaly_large += 1,
        }
    }
    
    println!("Results after {} additional samples:", larger_test);
    println!("ANOMALY={} EXCESS={} NORMAL={}", anomaly_large, excess_large, normal_large);
    println!("Z={:.15} T={:.15}", detector.anomaly_threshold(), detector.excess_threshold());
    
    Ok(())
}