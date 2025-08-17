//! Isolate the first excess case to debug the immediate Z divergence
//!
//! Focus on step 1028 where the first divergence occurs

use std::error::Error;
use libspot::{Spot, SpotConfig, SpotStatus}; 
use libspot_ffi::{SpotDetector, SpotStatus as FFISpotStatus};

// Import shared random number generation
use experiment::shared_random::*;

fn isolate_first_excess_divergence() -> Result<(), Box<dyn Error>> {
    println!("=== FIRST EXCESS DIVERGENCE ISOLATION ===");
    
    // Create both detectors
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
    
    // Generate identical training data
    set_random_seed(1);
    let training_data: Vec<f64> = (0..1000).map(|_| c_random()).collect();
    
    // Fit both detectors
    rust_spot.fit(&training_data)?;
    ffi_spot.fit(&training_data)?;
    
    println!("After fitting:");
    println!("  Both excess thresholds: {:.15}", rust_spot.excess_threshold());
    println!("  Initial Z values identical: {:.15}", rust_spot.quantile(0.0001));
    
    // Process exactly to step 1028 (the critical step)
    set_random_seed(1);
    for i in 1000..1028 {
        let x = c_random();
        rust_spot.step(x)?;
        ffi_spot.step(x)?;
    }
    
    // Now process step 1028 and examine in detail
    println!("\n=== CRITICAL STEP 1028 ===");
    let critical_x = c_random(); // This should be 0.998924517538399
    println!("Critical X value: {:.15}", critical_x);
    
    // Show pre-step state
    println!("Before step 1028:");
    println!("  Rust: nt={}, n={}, et={:.15}", rust_spot.nt(), rust_spot.n(), rust_spot.excess_threshold());
    println!("  FFI:  et={:.15}", ffi_spot.excess_threshold());
    
    // Process the critical step
    let rust_status = rust_spot.step(critical_x)?;
    let ffi_status = ffi_spot.step(critical_x)?;
    
    println!("After step 1028:");
    println!("  Status: Rust={:?}, FFI={:?}", rust_status, ffi_status);
    println!("  Rust: nt={}, n={}, et={:.15}", rust_spot.nt(), rust_spot.n(), rust_spot.excess_threshold());
    
    // Compare quantile calculation components
    let rust_z = rust_spot.quantile(0.0001);
    let ffi_z = ffi_spot.quantile(0.0001);
    
    println!("\n=== QUANTILE COMPONENTS ANALYSIS ===");
    println!("Final Z values:");
    println!("  Rust Z: {:.15}", rust_z);
    println!("  FFI Z:  {:.15}", ffi_z);
    println!("  Difference: {:.2e}", rust_z - ffi_z);
    
    // Break down the quantile calculation
    let rust_nt = rust_spot.nt() as f64;
    let rust_n = rust_spot.n() as f64;
    let rust_s = rust_nt / rust_n;
    let rust_et = rust_spot.excess_threshold();
    let (rust_gamma, rust_sigma) = rust_spot.tail_parameters();
    
    println!("\nRust quantile calculation breakdown:");
    println!("  nt={}, n={}, s=nt/n={:.15}", rust_spot.nt(), rust_spot.n(), rust_s);
    println!("  excess_threshold={:.15}", rust_et);
    println!("  gamma={:.15}, sigma={:.15}", rust_gamma, rust_sigma);
    
    // Manual quantile calculation for comparison
    // Z = excess_threshold + up_down * tail.quantile(s, q)
    // Since gamma == 0, tail.quantile(s, q) = -sigma * ln(q/s)
    let q = 0.0001;
    let r = q / rust_s;
    let tail_quantile = if rust_gamma == 0.0 {
        -rust_sigma * r.ln()
    } else {
        (rust_sigma / rust_gamma) * (r.powf(-rust_gamma) - 1.0)
    };
    
    let manual_z = rust_et + 1.0 * tail_quantile; // up_down = 1.0 for upper tail
    
    println!("  r=q/s={:.15}", r);
    println!("  tail_quantile=-sigma*ln(r)={:.15}", tail_quantile);
    println!("  manual_z=et+tail_quantile={:.15}", manual_z);
    println!("  Method match: {}", (manual_z - rust_z).abs() < 1e-15);
    
    // Show peaks data
    println!("\nPeaks data after first excess:");
    println!("  Size: {}", rust_spot.tail_size());
    println!("  Mean: {:.15}", rust_spot.peaks_mean());
    println!("  Min: {:.15}", rust_spot.peaks_min());
    println!("  Max: {:.15}", rust_spot.peaks_max());
    
    // The issue is likely in the tail parameters (gamma, sigma) calculation
    // Let's extract and compare the exact peaks data
    let peaks_data = rust_spot.peaks_data();
    println!("  Peaks values:");
    for (i, &peak) in peaks_data.iter().enumerate() {
        println!("    [{:2}] = {:.15}", i, peak);
    }
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    isolate_first_excess_divergence()?;
    Ok(())
}