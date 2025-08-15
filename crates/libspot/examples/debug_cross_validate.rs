//! Cross-validation debug test to isolate Grimshaw estimator differences
//!
//! This test implements the debugging approach to determine if the issue is in:
//! 1. Input data differences between pure Rust and FFI implementations
//! 2. Grimshaw estimator function implementation differences
//!
//! Strategy:
//! - Run both implementations in parallel
//! - Save excess values from pure Rust to file
//! - Feed Rust excess values to FFI Grimshaw estimator
//! - Compare all results to isolate the source of differences

use libspot::{Spot, SpotConfig, SpotStatus};
use std::fs::File;
use std::io::Write;

extern "C" {
    fn srand(seed: u32);
    fn rand() -> i32;
}

fn c_rand() -> f64 {
    unsafe { rand() as f64 / (i32::MAX as f64 + 1.0) }
}

fn save_excess_to_file(excess_values: &[f64], filename: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    for value in excess_values {
        writeln!(file, "{:.15}", value)?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== CROSS-VALIDATION DEBUG TEST ===");
    println!("Testing if input data or function implementation causes differences");
    
    // Use same seed as C implementation
    unsafe { srand(42) };
    
    let config = SpotConfig::default();
    let mut detector = Spot::new(config)?;
    
    // Generate and collect training data
    let mut training_data = Vec::with_capacity(20000);
    for _ in 0..20000 {
        training_data.push(c_rand());
    }
    
    // Fit the model
    detector.fit(&training_data)?;
    println!("Model fitted. Initial threshold T = {}", detector.excess_threshold());
    
    // Process samples until we have enough data to compare
    let target_step = 100000; // Test at the known divergence point
    let mut step_count = 0;
    let mut anomaly_count = 0;
    let mut excess_count = 0;
    let mut normal_count = 0;
    
    for _ in 0..target_step {
        let value = c_rand();
        let status = detector.step(value)?;
        step_count += 1;
        
        match status {
            SpotStatus::Anomaly => anomaly_count += 1,
            SpotStatus::Excess => excess_count += 1, 
            SpotStatus::Normal => normal_count += 1,
        }
    }
    
    println!("\n=== PURE RUST RESULTS AT STEP {} ===", step_count);
    let z_rust = detector.anomaly_threshold();
    let t_rust = detector.excess_threshold();
    let (gamma_rust, sigma_rust) = detector.get_gpd_parameters();
    let excess_values_rust = detector.get_excess_values();
    let (mean_rust, var_rust, min_rust, max_rust, len_rust) = detector.get_peaks_stats();
    
    println!("Z={:.9} T={:.9} Gamma={:.9} Sigma={:.9}", z_rust, t_rust, gamma_rust, sigma_rust);
    println!("Excess count: {}, Normal count: {}, Anomaly count: {}", excess_count, normal_count, anomaly_count);
    println!("Excess buffer: {} values, Mean={:.9}, Variance={:.9}", len_rust, mean_rust, var_rust);
    println!("Excess range: Min={:.9}, Max={:.9}", min_rust, max_rust);
    
    // Save the exact excess values from Rust to file
    let temp_dir = "/tmp";
    let rust_excess_file = format!("{}/rust_excess_values.txt", temp_dir);
    save_excess_to_file(&excess_values_rust, &rust_excess_file)?;
    println!("Saved {} excess values to {}", excess_values_rust.len(), rust_excess_file);
    
    // Sort excess values for point-by-point comparison
    let mut sorted_rust = excess_values_rust.clone();
    sorted_rust.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    // Call Rust Grimshaw estimator with the same data explicitly
    let (gamma_rust_manual, sigma_rust_manual, ll_rust_manual) = 
        detector.call_grimshaw_estimator_with_data(&excess_values_rust);
    
    println!("\n=== RUST GRIMSHAW ESTIMATOR (MANUAL CALL) ===");
    println!("Gamma={:.9} Sigma={:.9} LogLikelihood={:.9}", 
             gamma_rust_manual, sigma_rust_manual, ll_rust_manual);
    
    // Print first and last few values for manual verification
    println!("\n=== EXCESS VALUES DETAILS ===");
    println!("First 10 values: {:?}", &sorted_rust[..std::cmp::min(10, sorted_rust.len())]);
    if sorted_rust.len() > 10 {
        println!("Last 10 values: {:?}", &sorted_rust[sorted_rust.len().saturating_sub(10)..]);
    }
    
    // Calculate and print detailed statistics
    let sum: f64 = excess_values_rust.iter().sum();
    let sum_squares: f64 = excess_values_rust.iter().map(|x| x * x).sum();
    let n = excess_values_rust.len() as f64;
    let mean_calc = sum / n;
    let variance_calc = (sum_squares / n) - (mean_calc * mean_calc);
    
    println!("Statistical verification:");
    println!("  Count: {}", excess_values_rust.len());
    println!("  Sum: {:.15}", sum);
    println!("  Sum of squares: {:.15}", sum_squares);
    println!("  Mean (calculated): {:.15}", mean_calc);
    println!("  Variance (calculated): {:.15}", variance_calc);
    println!("  Min: {:.15}", sorted_rust[0]);
    println!("  Max: {:.15}", sorted_rust[sorted_rust.len() - 1]);
    
    println!("\n=== NEXT STEPS ===");
    println!("1. Run the corresponding FFI debug example");
    println!("2. Compare the excess values files point by point");
    println!("3. Feed the Rust excess values to FFI Grimshaw estimator");
    println!("4. Compare results to determine if issue is in input data or function implementation");
    
    Ok(())
}