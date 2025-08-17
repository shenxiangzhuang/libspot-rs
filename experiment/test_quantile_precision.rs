//! Isolate the exact mathematical difference in quantile calculations
//!
//! This experiment tests the exact same inputs through both quantile calculations
//! to find the numerical precision difference.

use std::error::Error;
use libspot::{Spot, SpotConfig};
use libspot_ffi::SpotDetector;

// Import shared random number generation
use experiment::shared_random::*;

fn test_quantile_precision_difference() -> Result<(), Box<dyn Error>> {
    println!("=== QUANTILE PRECISION ANALYSIS ===");
    
    // Recreate the exact state at the divergence point
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
    
    // Recreate exact state
    set_random_seed(1);
    let training_data: Vec<f64> = (0..1000).map(|_| c_random()).collect();
    rust_spot.fit(&training_data)?;
    ffi_spot.fit(&training_data)?;
    
    set_random_seed(1);
    for _i in 1000..1029 { // Process up to and including step 1028
        let x = c_random();
        rust_spot.step(x)?;
        ffi_spot.step(x)?;
    }
    
    // Now both are in the exact state where divergence occurs
    println!("State after step 1028:");
    println!("  nt={}, n={}", rust_spot.nt(), rust_spot.n());
    println!("  excess_threshold={:.15}", rust_spot.excess_threshold());
    println!("  s=nt/n={:.15}", rust_spot.nt() as f64 / rust_spot.n() as f64);
    
    let (gamma, sigma) = rust_spot.tail_parameters();
    println!("  gamma={:.15}, sigma={:.15}", gamma, sigma);
    
    // Test multiple quantile values to see if the pattern is consistent
    let test_quantiles = [0.0001, 0.001, 0.01, 0.1];
    
    for q in test_quantiles.iter() {
        let rust_z = rust_spot.quantile(*q);
        let ffi_z = ffi_spot.quantile(*q);
        let diff = rust_z - ffi_z;
        
        println!("q={:.4}: Rust={:.15}, FFI={:.15}, diff={:.2e}", q, rust_z, ffi_z, diff);
    }
    
    // Test the mathematical components directly
    println!("\n=== MATHEMATICAL COMPONENT BREAKDOWN ===");
    let q = 0.0001;
    let s = rust_spot.nt() as f64 / rust_spot.n() as f64;
    let r = q / s;
    let et = rust_spot.excess_threshold();
    
    println!("Input parameters:");
    println!("  q={:.15}", q);
    println!("  s={:.15}", s);
    println!("  r=q/s={:.15}", r);
    println!("  sigma={:.15}", sigma);
    println!("  excess_threshold={:.15}", et);
    
    // Test different ln implementations
    let ln_r_std = r.ln();
    
    println!("\nLogarithm comparisons:");
    println!("  Rust r.ln()={:.15}", ln_r_std);
    
    // Manual quantile calculations
    let tail_quantile_std = -sigma * ln_r_std;
    
    let z_std = et + tail_quantile_std;
    
    println!("\nTail quantile calculations:");
    println!("  With Rust ln(): {:.15}", tail_quantile_std);
    
    println!("\nFinal Z calculations:");
    println!("  With Rust ln(): {:.15}", z_std);
    
    // Compare with actual results
    let actual_rust_z = rust_spot.quantile(0.0001);
    let actual_ffi_z = ffi_spot.quantile(0.0001);
    
    println!("\nActual implementation results:");
    println!("  Actual Rust:    {:.15}", actual_rust_z);
    println!("  Actual FFI:     {:.15}", actual_ffi_z);
    println!("  Actual diff:    {:.2e}", actual_rust_z - actual_ffi_z);
    
    println!("\nHypothesis verification:");
    println!("  Rust matches std ln: {}", (actual_rust_z - z_std).abs() < 1e-15);
    
    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    test_quantile_precision_difference()?;
    Ok(())
}