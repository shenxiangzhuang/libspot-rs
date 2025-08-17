//! Find the exact divergence point by comparing Z values after each excess
//!
//! This experiment tracks Z values after every excess to find where they diverge.

use std::error::Error;
use std::fs::File; 
use std::io::Write;
use libspot::{Spot, SpotConfig, SpotStatus};
use libspot_ffi::{SpotDetector, SpotStatus as FFISpotStatus};

// Import shared random number generation
use experiment::shared_random::*;

/// Find the exact step where Z values start diverging
fn find_z_divergence_point() -> Result<(), Box<dyn Error>> {
    println!("=== Z VALUE DIVERGENCE TRACKING ===");
    
    // Set up both detectors with identical configuration
    let rust_config = SpotConfig {
        q: 0.0001,
        level: 0.998,
        low_tail: false,
        discard_anomalies: true,
        max_excess: 200,
    };
    
    let ffi_config = libspot_ffi::SpotConfig {
        q: 0.0001,
        level: 0.998,
        low_tail: false,
        discard_anomalies: true,
        max_excess: 200,
    };
    
    let mut rust_spot = Spot::new(rust_config)?;
    let mut ffi_spot = SpotDetector::new(ffi_config)?;
    
    // Generate the same initial training data
    set_random_seed(1);
    let training_data: Vec<f64> = (0..1000).map(|_| c_random()).collect();
    
    // Fit both detectors
    rust_spot.fit(&training_data)?;
    ffi_spot.fit(&training_data)?;
    
    println!("Initial Z values after fitting:");
    let initial_rust_z = rust_spot.quantile(0.0001);
    let initial_ffi_z = ffi_spot.quantile(0.0001);
    println!("  Rust Z: {:.15}", initial_rust_z);
    println!("  FFI Z:  {:.15}", initial_ffi_z);
    println!("  Diff:   {:.2e}", initial_rust_z - initial_ffi_z);
    
    // Track Z updates and find divergence
    let mut z_update_count = 0;
    let mut log_file = File::create("z_divergence_tracking.txt")?;
    writeln!(log_file, "step,z_update,x,rust_z,ffi_z,z_diff,rust_nt,rust_n,et_diff")?;
    
    set_random_seed(1);
    for i in 1000..100000 { // Go up to 100k steps to find the divergence
        let x = c_random();
        
        let rust_status = rust_spot.step(x)?;
        let ffi_status = ffi_spot.step(x)?;
        
        // Both should have same excess status (we verified this above)
        if matches!(rust_status, SpotStatus::Excess) {
            z_update_count += 1;
            
            let rust_z = rust_spot.quantile(0.0001);
            let ffi_z = ffi_spot.quantile(0.0001);
            let z_diff = rust_z - ffi_z;
            let rust_et = rust_spot.excess_threshold();
            let ffi_et = ffi_spot.excess_threshold();
            let et_diff = rust_et - ffi_et;
            
            // Log this Z update
            writeln!(log_file, "{},{},{:.15},{:.15},{:.15},{:.2e},{},{},{:.2e}", 
                     i, z_update_count, x, rust_z, ffi_z, z_diff, 
                     rust_spot.nt(), rust_spot.n(), et_diff)?;
            
            // Check for significant divergence
            if z_diff.abs() > 1e-10 {
                println!("*** DIVERGENCE DETECTED ***");
                println!("Step: {}, Z Update: {}, X: {:.15}", i, z_update_count, x);
                println!("  Rust Z: {:.15}", rust_z);
                println!("  FFI Z:  {:.15}", ffi_z);
                println!("  Z Diff: {:.2e}", z_diff);
                println!("  ET Diff: {:.2e}", et_diff);
                
                // Show tail parameters
                let (rust_gamma, rust_sigma) = rust_spot.tail_parameters();
                println!("  Rust: gamma={:.15}, sigma={:.15}", rust_gamma, rust_sigma);
                println!("  Rust: nt={}, n={}", rust_spot.nt(), rust_spot.n());
                
                // Show peaks statistics
                println!("  Rust peaks: size={}, mean={:.15}, min={:.15}, max={:.15}", 
                         rust_spot.tail_size(), rust_spot.peaks_mean(), 
                         rust_spot.peaks_min(), rust_spot.peaks_max());
                
                break;
            }
            
            // Print periodic updates
            if z_update_count % 10 == 0 {
                println!("Z Update {}: Step={}, Z_diff={:.2e}", z_update_count, i, z_diff);
            }
        }
    }
    
    log_file.flush()?;
    println!("Divergence tracking completed. Check z_divergence_tracking.txt for details.");
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    find_z_divergence_point()?;
    Ok(())
}