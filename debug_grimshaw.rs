//! Debug the Grimshaw estimator at the exact divergence point
//! This will help identify if the issue is in the tail fitting logic

use libspot::{Spot, SpotConfig, SpotStatus, Peaks};
use libspot_ffi::{SpotDetector, SpotStatus as FFIStatus, SpotConfig as FFIConfig};
use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut debug_file = File::create("/tmp/grimshaw_debug.txt")?;
    
    println!("Debugging Grimshaw estimator at divergence point...");
    
    // Set up exactly the same configuration
    let rust_config = SpotConfig {
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
    
    let mut rust_detector = Spot::new(rust_config)?;
    let mut ffi_detector = SpotDetector::new(ffi_config)?;
    
    // Generate the exact same sequence that led to divergence
    let mut rng = CRand::new(1);
    
    // Initial training data (20000 samples)
    let n = 20000;
    let mut initial_data = Vec::with_capacity(n);
    
    for _ in 0..n {
        initial_data.push(rng.rexp());
    }
    
    // Fit both models
    rust_detector.fit(&initial_data)?;
    ffi_detector.fit(&initial_data)?;
    
    writeln!(debug_file, "After initial fit:")?;
    writeln!(debug_file, "Rust: Z={:.15} T={:.15}", 
             rust_detector.anomaly_threshold(), rust_detector.excess_threshold())?;
    writeln!(debug_file, "FFI:  Z={:.15} T={:.15}", 
             ffi_detector.anomaly_threshold(), ffi_detector.excess_threshold())?;
    writeln!(debug_file, "")?;
    
    // Process exactly the steps that led to the divergence (97066 steps)
    let mut excess_count = 0;
    for step in 1..=97066 {
        let val = rng.rexp();
        
        let rust_status = rust_detector.step(val)?;
        let ffi_status = ffi_detector.step(val)?;
        
        if matches!(rust_status, SpotStatus::Excess) || matches!(ffi_status, FFIStatus::Excess) {
            excess_count += 1;
            
            // Around the critical point (update 165-166)
            if excess_count >= 165 && excess_count <= 167 {
                writeln!(debug_file, "=== CRITICAL UPDATE {} at step {} ===", excess_count, step)?;
                writeln!(debug_file, "Input value: {:.15}", val)?;
                writeln!(debug_file, "Excess value: {:.15}", val - rust_detector.excess_threshold())?;
                
                // Get tail parameters
                let (rust_gamma, rust_sigma) = rust_detector.tail_parameters();
                writeln!(debug_file, "Rust: Status={:?} Z={:.15} Gamma={:.15} Sigma={:.15}", 
                         rust_status, rust_detector.anomaly_threshold(), rust_gamma, rust_sigma)?;
                         
                writeln!(debug_file, "FFI:  Status={:?} Z={:.15}", 
                         ffi_status, ffi_detector.anomaly_threshold())?;
                         
                let z_diff = (rust_detector.anomaly_threshold() - ffi_detector.anomaly_threshold()).abs();
                writeln!(debug_file, "Z Difference: {:.15}", z_diff)?;
                
                // Log peaks data state
                writeln!(debug_file, "Tail size: {} Mean: {:.15} Min: {:.15} Max: {:.15}", 
                         rust_detector.tail_size(),
                         rust_detector.peaks_mean(),
                         rust_detector.peaks_min(),
                         rust_detector.peaks_max())?;
                writeln!(debug_file, "")?;
                
                // Stop after the divergence
                if z_diff > 1e-10 {
                    break;
                }
            }
        }
    }
    
    writeln!(debug_file, "Debug completed. Total excess updates: {}", excess_count)?;
    println!("Debug output written to /tmp/grimshaw_debug.txt");
    
    Ok(())
}

/// Random number generator matching C's rand()/srand()
struct CRand;

impl CRand {
    fn new(seed: u32) -> Self {
        unsafe { libc::srand(seed); }
        CRand
    }
    
    fn rand(&mut self) -> u32 {
        unsafe { libc::rand() as u32 }
    }
    
    fn runif(&mut self) -> f64 {
        self.rand() as f64 / 2147483647.0
    }
    
    fn rexp(&mut self) -> f64 {
        -self.runif().ln()
    }
}