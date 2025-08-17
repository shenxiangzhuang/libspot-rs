//! Test to compare intermediate values in quantile calculation

use std::error::Error;
use libspot::{Spot, SpotConfig};
use libspot_ffi::{SpotDetector, ffi};

// Import shared random number generation from experiment
use experiment::shared_random::*;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== INTERMEDIATE VALUES COMPARISON ===");
    
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
    
    println!("State after step 1028:");
    
    // Compare all intermediate values
    println!("=== BASIC COUNTS ===");
    println!("Rust nt: {}, n: {}", rust_spot.nt(), rust_spot.n());
    println!("FFI  nt: {}, n: {}", ffi_spot.nt(), ffi_spot.n());
    println!("nt difference: {}", ffi_spot.nt() as i64 - rust_spot.nt() as i64);
    println!("n difference:  {}", ffi_spot.n() as i64 - rust_spot.n() as i64);
    
    println!("\n=== S VALUES (nt/n) ===");
    let rust_s = rust_spot.nt() as f64 / rust_spot.n() as f64;
    let ffi_s = ffi_spot.nt() as f64 / ffi_spot.n() as f64;
    println!("Rust s = nt/n = {:.15}", rust_s);
    println!("FFI  s = nt/n = {:.15}", ffi_s);
    println!("S difference: {:.2e}", ffi_s - rust_s);
    
    println!("\n=== EXCESS THRESHOLDS ===");
    println!("Rust ET: {:.15}", rust_spot.excess_threshold());
    println!("FFI  ET: {:.15}", ffi_spot.excess_threshold());
    println!("ET difference: {:.2e}", ffi_spot.excess_threshold() - rust_spot.excess_threshold());
    
    println!("\n=== TAIL PARAMETERS ===");
    let (rust_gamma, rust_sigma) = rust_spot.tail_parameters();
    let (ffi_gamma, ffi_sigma) = ffi_spot.tail_parameters();
    println!("Rust gamma: {:.15}, sigma: {:.15}", rust_gamma, rust_sigma);
    println!("FFI  gamma: {:.15}, sigma: {:.15}", ffi_gamma, ffi_sigma);
    println!("Gamma difference: {:.2e}", ffi_gamma - rust_gamma);
    println!("Sigma difference: {:.2e}", ffi_sigma - rust_sigma);
    
    println!("\n=== QUANTILE CALCULATION BREAKDOWN ===");
    let q = 0.0001;
    let rust_r = q / rust_s;
    let ffi_r = q / ffi_s;
    println!("Rust r = q/s = {:.15}", rust_r);
    println!("FFI  r = q/s = {:.15}", ffi_r);
    println!("R difference: {:.2e}", ffi_r - rust_r);
    
    // Manual quantile calculations
    let rust_tail_quantile = if rust_gamma == 0.0 {
        -rust_sigma * libspot::math::xlog(rust_r)
    } else {
        (rust_sigma / rust_gamma) * (libspot::math::xpow(rust_r, -rust_gamma) - 1.0)
    };
    
    let ffi_tail_quantile = if ffi_gamma == 0.0 {
        -ffi_sigma * unsafe { libspot_ffi::ffi::xlog(ffi_r) }
    } else {
        (ffi_sigma / ffi_gamma) * (unsafe { libspot_ffi::ffi::xpow(ffi_r, -ffi_gamma) } - 1.0)
    };
    
    println!("Rust tail quantile: {:.15}", rust_tail_quantile);
    println!("FFI  tail quantile: {:.15}", ffi_tail_quantile);
    println!("Tail quantile difference: {:.2e}", ffi_tail_quantile - rust_tail_quantile);
    
    let rust_final_z = rust_spot.excess_threshold() + rust_tail_quantile;
    let ffi_final_z = ffi_spot.excess_threshold() + ffi_tail_quantile;
    
    println!("\nFinal Z calculations:");
    println!("Rust final Z: {:.15}", rust_final_z);
    println!("FFI  final Z: {:.15}", ffi_final_z);
    println!("Final Z difference: {:.2e}", ffi_final_z - rust_final_z);
    println!("Rust final Z: {:.15}", rust_final_z);
    
    // Compare with actual quantile function results
    println!("\n=== ACTUAL QUANTILE FUNCTION RESULTS ===");
    let actual_rust_z = rust_spot.quantile(q);
    let actual_ffi_z = ffi_spot.quantile(q);
    println!("Actual Rust Z: {:.15}", actual_rust_z);
    println!("Actual FFI  Z: {:.15}", actual_ffi_z);
    println!("Actual difference: {:.2e}", actual_ffi_z - actual_rust_z);
    
    println!("\n=== VERIFICATION ===");
    println!("Manual Rust matches actual: {}", (rust_final_z - actual_rust_z).abs() < 1e-15);
    println!("Manual FFI matches actual:  {}", (ffi_final_z - actual_ffi_z).abs() < 1e-15);
    
    // The key insight: if ET is identical but final quantiles differ,
    // the difference must be in the tail quantile calculation
    let implied_ffi_tail_quantile = actual_ffi_z - ffi_spot.excess_threshold();
    println!("Implied FFI tail quantile: {:.15}", implied_ffi_tail_quantile);
    println!("Tail quantile difference:   {:.2e}", implied_ffi_tail_quantile - rust_tail_quantile);
    
    // This difference in tail quantile calculation is where the bug is!
    // Let's analyze what could cause this difference
    println!("\n=== ROOT CAUSE ANALYSIS ===");
    println!("Since:");
    println!("- Excess thresholds are identical: {:.2e}", ffi_spot.excess_threshold() - rust_spot.excess_threshold());
    println!("- xlog functions are identical");
    println!("- The difference is in tail quantile: {:.2e}", implied_ffi_tail_quantile - rust_tail_quantile);
    println!("");
    println!("The issue must be in one of:");
    println!("1. Different s values (nt/n ratios)");
    println!("2. Different gamma/sigma parameters");
    println!("3. Different calculation logic in quantile function");
    
    Ok(())
}