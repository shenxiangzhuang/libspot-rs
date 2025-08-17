//! Isolate different SPOT algorithm components to find the exact divergence
//!
//! This experiment isolates the quantile calculation logic to compare
//! the pure Rust and C FFI implementations component by component.

use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use libspot::spot::Spot;
use libspot::config::SpotConfig;
use libspot_ffi::detector::SpotDetector;

// Import shared random number generation
use crate::shared_random::*;

/// Test the quantile calculation with the exact same parameters
fn test_quantile_calculation() -> Result<(), Box<dyn Error>> {
    println!("=== QUANTILE CALCULATION COMPARISON ===");
    
    // Load the critical peaks data
    let peaks_data = load_critical_peaks_data()?;
    println!("Loaded {} peaks values", peaks_data.len());
    
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
    
    // Process samples until we reach the critical point (step 97066, 166th Z update)
    set_random_seed(1);
    let mut step_count = 0;
    let mut z_update_count = 0;
    
    for i in 1000..97066 {
        let x = c_random();
        
        let rust_status = rust_spot.step(x)?;
        let ffi_status = ffi_spot.step(x)?;
        
        // Count Z updates (when we get an excess)
        if matches!(rust_status, libspot::status::SpotStatus::Excess) {
            z_update_count += 1;
            
            if z_update_count >= 160 { // Near the critical point
                let rust_z = rust_spot.quantile(0.0001);
                let ffi_z = ffi_spot.quantile(0.0001);
                
                println!("Z Update {}: Rust Z={:.15}, FFI Z={:.15}, Diff={:.2e}", 
                         z_update_count, rust_z, ffi_z, rust_z - ffi_z);
                
                // Also isolate the internal parameters
                let rust_nt = rust_spot.nt();
                let rust_n = rust_spot.n();
                let rust_et = rust_spot.excess_threshold();
                let ffi_et = ffi_spot.excess_threshold();
                let (rust_gamma, rust_sigma) = rust_spot.tail_parameters();
                
                println!("  Rust: nt={}, n={}, et={:.15}, gamma={:.15}, sigma={:.15}", 
                         rust_nt, rust_n, rust_et, rust_gamma, rust_sigma);
                println!("  FFI:  et={:.15}", ffi_et);
                
                // Test the components separately
                let s_rust = rust_nt as f64 / rust_n as f64;
                let tail_quantile_rust = rust_spot.tail.quantile(s_rust, 0.0001);
                let final_z_rust = rust_et + 1.0 * tail_quantile_rust; // up_down = 1.0 for upper tail
                
                println!("  Manual Rust calculation: s={:.15}, tail_quantile={:.15}, final_z={:.15}", 
                         s_rust, tail_quantile_rust, final_z_rust);
            }
            
            if z_update_count >= 166 {
                break;
            }
        }
        
        step_count += 1;
    }
    
    println!("\n=== FINAL CRITICAL COMPARISON ===");
    let rust_z_final = rust_spot.quantile(0.0001);
    let ffi_z_final = ffi_spot.quantile(0.0001);
    
    println!("Final Rust Z: {:.15}", rust_z_final);
    println!("Final FFI Z:  {:.15}", ffi_z_final);
    println!("Difference:   {:.15}", rust_z_final - ffi_z_final);
    
    Ok(())
}

/// Load the critical peaks data from CSV
fn load_critical_peaks_data() -> Result<Vec<f64>, Box<dyn Error>> {
    let file = File::open("critical_peaks_data.csv")?;
    let reader = BufReader::new(file);
    let mut peaks = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        if let Ok(value) = line.trim().parse::<f64>() {
            peaks.push(value);
        }
    }
    
    Ok(peaks)
}

fn main() -> Result<(), Box<dyn Error>> {
    test_quantile_calculation()?;
    Ok(())
}