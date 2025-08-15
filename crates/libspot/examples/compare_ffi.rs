use libspot::{Spot, SpotConfig, SpotStatus};
use libspot_ffi::{SpotDetector, SpotConfig as FFIConfig, SpotStatus as FFIStatus};
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
    // Set debug for rust implementation
    env::set_var("SPOT_DEBUG_GRIMSHAW", "1");
    
    let config = SpotConfig {
        q: 0.0001,
        low_tail: false,
        discard_anomalies: true,
        level: 0.998,
        max_excess: 200,
    };
    
    let ffi_config = FFIConfig {
        q: 0.0001,
        low_tail: false,
        discard_anomalies: true,
        level: 0.998,
        max_excess: 200,
    };

    let mut rust_detector = Spot::new(config)?;
    let mut ffi_detector = SpotDetector::new(ffi_config)?;
    let mut rng = CRand::new(1);

    // Fit both models
    let n = 20000;
    let mut initial_data = Vec::with_capacity(n);
    for _ in 0..n {
        initial_data.push(rng.rexp());
    }
    
    rust_detector.fit(&initial_data)?;
    ffi_detector.fit(&initial_data)?;

    println!("After fit:");
    println!("Rust: Z={:.15} T={:.15}", rust_detector.anomaly_threshold(), rust_detector.excess_threshold());
    println!("FFI:  Z={:.15} T={:.15}", ffi_detector.anomaly_threshold(), ffi_detector.excess_threshold());

    // Compare step by step for first 100k samples
    let mut rust_anomaly = 0;
    let mut rust_excess = 0;
    let mut rust_normal = 0;
    let mut ffi_anomaly = 0;
    let mut ffi_excess = 0;
    let mut ffi_normal = 0;

    for i in 0..100000 {
        let val = rng.rexp();
        
        match rust_detector.step(val)? {
            SpotStatus::Normal => rust_normal += 1,
            SpotStatus::Excess => rust_excess += 1,
            SpotStatus::Anomaly => rust_anomaly += 1,
        }
        
        match ffi_detector.step(val)? {
            FFIStatus::Normal => ffi_normal += 1,
            FFIStatus::Excess => ffi_excess += 1,
            FFIStatus::Anomaly => ffi_anomaly += 1,
        }

        // Check at key intervals
        if i == 9999 || i == 49999 || i == 99999 {
            println!("\nAt {} samples:", i + 1);
            println!("Rust: ANOMALY={} EXCESS={} NORMAL={} Z={:.15} T={:.15}", 
                     rust_anomaly, rust_excess, rust_normal,
                     rust_detector.anomaly_threshold(), rust_detector.excess_threshold());
            println!("FFI:  ANOMALY={} EXCESS={} NORMAL={} Z={:.15} T={:.15}", 
                     ffi_anomaly, ffi_excess, ffi_normal,
                     ffi_detector.anomaly_threshold(), ffi_detector.excess_threshold());
            
            let z_diff = (rust_detector.anomaly_threshold() - ffi_detector.anomaly_threshold()).abs();
            let t_diff = (rust_detector.excess_threshold() - ffi_detector.excess_threshold()).abs();
            println!("Diffs: Z_diff={:.15} T_diff={:.15}", z_diff, t_diff);
        }
    }

    Ok(())
}