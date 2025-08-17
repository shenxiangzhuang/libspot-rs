//! Test script to directly call the Grimshaw estimators and compare results

use libspot::{Peaks, Ubend};
// use libspot_ffi::ffi;
use std::fs::File;
use std::io::Write;
use std::os::raw::c_double;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut debug_file = File::create("/tmp/grimshaw_direct.txt")?;
    
    println!("Testing Grimshaw estimators directly...");
    
    // Recreate the exact peaks data at the divergence point
    // We know from the debug output that after 165 updates:
    // - Mean: 0.764398907444766
    // - Min: 0.003817830505861  
    // - Max: 3.813152398048215
    // - Size: 200
    
    // Let me first run the full sequence to get the exact peaks data
    let mut rng = CRand::new(1);
    
    // Skip to training data
    let n = 20000;
    for _ in 0..n {
        rng.rexp();
    }
    
    // Process the steps leading up to divergence
    let excess_threshold = 6.236165177550786;
    let mut peaks = Peaks::new(200)?;
    let mut step_count = 0;
    let mut excess_count = 0;
    
    loop {
        let val = rng.rexp();
        step_count += 1;
        
        let excess = val - excess_threshold;
        if excess >= 0.0 {
            peaks.push(excess);
            excess_count += 1;
            
            // Stop right at the point where we're about to add the diverging excess
            if excess_count == 165 {
                writeln!(debug_file, "Peaks data after 165 excesses:")?;
                writeln!(debug_file, "Size: {} Mean: {:.15} Min: {:.15} Max: {:.15}", 
                         peaks.size(), peaks.mean(), peaks.min(), peaks.max())?;
                         
                // Call both Grimshaw estimators on this data
                let (rust_gamma, rust_sigma, rust_ll) = libspot::estimator::grimshaw_estimator(&peaks);
                writeln!(debug_file, "Rust Grimshaw: Gamma={:.15} Sigma={:.15} LL={:.15}", 
                         rust_gamma, rust_sigma, rust_ll)?;
                
                // Now add the diverging excess
                let diverging_excess = 7.099368295593661 - excess_threshold;
                peaks.push(diverging_excess);
                writeln!(debug_file, "\nAfter adding diverging excess: {:.15}", diverging_excess)?;
                writeln!(debug_file, "Size: {} Mean: {:.15} Min: {:.15} Max: {:.15}", 
                         peaks.size(), peaks.mean(), peaks.min(), peaks.max())?;
                         
                // Call both Grimshaw estimators on the updated data
                let (rust_gamma2, rust_sigma2, rust_ll2) = libspot::estimator::grimshaw_estimator(&peaks);
                writeln!(debug_file, "Rust Grimshaw: Gamma={:.15} Sigma={:.15} LL={:.15}", 
                         rust_gamma2, rust_sigma2, rust_ll2)?;
                
                break;
            }
        }
    }
    
    writeln!(debug_file, "Direct test completed.")?;
    println!("Debug output written to /tmp/grimshaw_direct.txt");
    
    Ok(())
}

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