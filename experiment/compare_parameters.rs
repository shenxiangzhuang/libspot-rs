//! Parameter comparison to isolate where SPOT diverges
//!
//! This experiment compares the exact parameter calculations (nt, n, s)
//! between pure Rust and C FFI implementations to find the divergence.

use std::error::Error;
use std::fs::File;
use std::io::Write;
use libspot::spot::Spot;
use libspot::config::SpotConfig;
use libspot_ffi::detector::SpotDetector;

// Import shared random number generation  
use crate::shared_random::*;

/// Compare parameters step by step to find where they diverge
fn compare_parameters_step_by_step() -> Result<(), Box<dyn Error>> {
    println!("=== PARAMETER COMPARISON ===");
    
    // Set up both detectors with identical configuration
    let config = SpotConfig {
        q: 0.0001,
        level: 0.998, 
        low_tail: false,
        discard_anomalies: true,
        max_excess: 200,
    };
    
    // Create both implementations
    let mut rust_spot = Spot::new(config.clone())?;
    let mut ffi_spot = SpotDetector::new(config.clone())?;
    
    // Generate the same initial training data
    set_random_seed(1);
    let training_data: Vec<f64> = (0..1000).map(|_| c_random()).collect();
    
    // Fit both detectors
    rust_spot.fit(&training_data)?;
    ffi_spot.fit(&training_data)?;
    
    // Check parameters after fitting
    println!("After fitting:");
    println!("  Rust: nt={}, n={}, et={:.15}", rust_spot.nt(), rust_spot.n(), rust_spot.excess_threshold());
    println!("  FFI:  et={:.15}", ffi_spot.excess_threshold());
    
    // Create log file for detailed parameter tracking
    let mut log_file = File::create("parameter_comparison.txt")?;
    writeln!(log_file, "step,rust_nt,rust_n,rust_s,ffi_et,rust_et,rust_z,ffi_z,x_value")?;
    
    // Process samples step by step until the critical point
    set_random_seed(1);
    let mut z_update_count = 0;
    
    for i in 1000..98000 { // Go a bit beyond the critical step 97066
        let x = c_random();
        
        // Store before processing
        let rust_nt_before = rust_spot.nt();
        let rust_n_before = rust_spot.n();
        
        let rust_status = rust_spot.step(x)?;
        let ffi_status = ffi_spot.step(x)?;
        
        // Check if both detected the same status
        let rust_is_excess = matches!(rust_status, libspot::status::SpotStatus::Excess);
        let ffi_is_excess = matches!(ffi_status, libspot_ffi::status::SpotStatus::Excess);
        
        if rust_is_excess != ffi_is_excess {
            println!("STATUS DIVERGENCE at step {}: Rust={:?}, FFI={:?}", i, rust_status, ffi_status);
            break;
        }
        
        if rust_is_excess {
            z_update_count += 1;
            
            let rust_nt = rust_spot.nt();
            let rust_n = rust_spot.n();
            let rust_s = rust_nt as f64 / rust_n as f64;
            let rust_et = rust_spot.excess_threshold();
            let ffi_et = ffi_spot.excess_threshold();
            let rust_z = rust_spot.quantile(0.0001);
            let ffi_z = ffi_spot.quantile(0.0001);
            
            // Log detailed parameters
            writeln!(log_file, "{},{},{},{:.15},{:.15},{:.15},{:.15},{:.15},{:.15}", 
                     i, rust_nt, rust_n, rust_s, ffi_et, rust_et, rust_z, ffi_z, x)?;
            
            // Check for parameter differences
            let et_diff = (rust_et - ffi_et).abs();
            let z_diff = (rust_z - ffi_z).abs();
            
            if z_update_count >= 160 || et_diff > 1e-10 || z_diff > 1e-10 {
                println!("Update {}: step={}, x={:.15}", z_update_count, i, x);
                println!("  Rust: nt={}, n={}, s={:.15}, et={:.15}, z={:.15}", 
                         rust_nt, rust_n, rust_s, rust_et, rust_z);
                println!("  FFI:  et={:.15}, z={:.15}", ffi_et, ffi_z);
                println!("  Diffs: et={:.2e}, z={:.2e}", et_diff, z_diff);
                
                if et_diff > 1e-10 {
                    println!("  *** EXCESS THRESHOLD DIVERGENCE DETECTED! ***");
                    break;
                }
                
                if z_update_count >= 170 {
                    break;
                }
            }
        }
    }
    
    log_file.flush()?;
    println!("Parameter comparison completed. Check parameter_comparison.txt for details.");
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    compare_parameters_step_by_step()?;
    Ok(())
}