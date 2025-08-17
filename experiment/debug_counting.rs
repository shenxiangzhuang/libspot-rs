//! Debug the exact n and nt counting differences between implementations  
//!
//! This isolates the step counting logic to see if there are differences
//! in how data points are counted or classified.

use std::error::Error;
use std::fs::File;  
use std::io::Write;
use libspot::{Spot, SpotConfig, SpotStatus};
use libspot_ffi::{SpotDetector, SpotStatus as FFISpotStatus};

// Import shared random number generation
use experiment::shared_random::*;

/// Test whether both implementations count data points identically
fn debug_counting_logic() -> Result<(), Box<dyn Error>> {
    println!("=== COUNTING LOGIC DEBUG ===");
    
    // Set up both detectors with identical configuration
    
    // Create both implementations  
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
    
    // Track anomaly counts
    let mut rust_anomaly_count = 0;
    let mut rust_excess_count = 0;
    let mut rust_normal_count = 0;
    let mut ffi_anomaly_count = 0;
    let mut ffi_excess_count = 0;
    let mut ffi_normal_count = 0;
    
    // Create detailed step log
    let mut log_file = File::create("counting_debug.txt")?;
    writeln!(log_file, "step,x,rust_status,ffi_status,rust_n,rust_nt,rust_anom,rust_exc,rust_norm,ffi_anom,ffi_exc,ffi_norm")?;
    
    // Process samples and track counting differences
    set_random_seed(1);
    for i in 1000..50000 { // Check first 50k steps thoroughly
        let x = c_random();
        
        let rust_status = rust_spot.step(x)?;
        let ffi_status = ffi_spot.step(x)?;
        
        // Update counts
        match rust_status {
            SpotStatus::Anomaly => rust_anomaly_count += 1,
            SpotStatus::Excess => rust_excess_count += 1,
            SpotStatus::Normal => rust_normal_count += 1,
        }
        
        match ffi_status {
            FFISpotStatus::Anomaly => ffi_anomaly_count += 1,
            FFISpotStatus::Excess => ffi_excess_count += 1,
            FFISpotStatus::Normal => ffi_normal_count += 1,
        }
        
        // Check for status mismatches
        let status_match = match (rust_status, ffi_status) {
            (SpotStatus::Anomaly, FFISpotStatus::Anomaly) => true,
            (SpotStatus::Excess, FFISpotStatus::Excess) => true,
            (SpotStatus::Normal, FFISpotStatus::Normal) => true,
            _ => false,
        };
        
        if !status_match {
            println!("STATUS MISMATCH at step {}: Rust={:?}, FFI={:?}, x={:.15}", 
                     i, rust_status, ffi_status, x);
            println!("  Rust counts: A={}, E={}, N={}", rust_anomaly_count, rust_excess_count, rust_normal_count);
            println!("  FFI counts:  A={}, E={}, N={}", ffi_anomaly_count, ffi_excess_count, ffi_normal_count);
            break;
        }
        
        // Log every step for detailed analysis
        if i % 1000 == 0 || i < 2000 {
            writeln!(log_file, "{},{:.15},{:?},{:?},{},{},{},{},{},{},{},{}",
                     i, x, rust_status, ffi_status, 
                     rust_spot.n(), rust_spot.nt(),
                     rust_anomaly_count, rust_excess_count, rust_normal_count,
                     ffi_anomaly_count, ffi_excess_count, ffi_normal_count)?;
        }
        
        // Check internal counter consistency
        let rust_n = rust_spot.n();
        let rust_nt = rust_spot.nt();
        let expected_total = 1000 + rust_normal_count + rust_excess_count; // Training + processed non-anomalies
        
        if rust_n != expected_total {
            println!("COUNTER INCONSISTENCY at step {}: rust_n={}, expected={}", i, rust_n, expected_total);
            println!("  Training: 1000, Normal: {}, Excess: {}, Anomaly: {}", 
                     rust_normal_count, rust_excess_count, rust_anomaly_count);
            break;
        }
    }
    
    log_file.flush()?;
    
    println!("Final counts:");
    println!("  Rust: Anomaly={}, Excess={}, Normal={}", rust_anomaly_count, rust_excess_count, rust_normal_count);
    println!("  FFI:  Anomaly={}, Excess={}, Normal={}", ffi_anomaly_count, ffi_excess_count, ffi_normal_count);
    println!("  Rust internal: n={}, nt={}", rust_spot.n(), rust_spot.nt());
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    debug_counting_logic()?;
    Ok(())
}