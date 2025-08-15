//! Cross-validation debug test for FFI implementation
//!
//! This test:
//! 1. Runs FFI implementation to generate its own excess values
//! 2. Loads the excess values saved by the Rust implementation
//! 3. Feeds both datasets to the FFI Grimshaw estimator
//! 4. Compares results to isolate where differences occur

use libspot_ffi::{SpotDetector, SpotConfig, SpotStatus};
use std::fs::File;
use std::io::{BufRead, BufReader};

extern "C" {
    fn srand(seed: u32);
    fn rand() -> i32;
}

fn c_rand() -> f64 {
    unsafe { rand() as f64 / (i32::MAX as f64 + 1.0) }
}

fn load_excess_from_file(filename: &str) -> std::io::Result<Vec<f64>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut values = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        let value: f64 = line.trim().parse().map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, e)
        })?;
        values.push(value);
    }
    
    Ok(values)
}

fn save_excess_to_file(excess_values: &[f64], filename: &str) -> std::io::Result<()> {
    let mut file = File::create(filename)?;
    use std::io::Write;
    for value in excess_values {
        writeln!(file, "{:.15}", value)?;
    }
    Ok(())
}

fn compare_arrays_detailed(arr1: &[f64], arr2: &[f64], name1: &str, name2: &str) {
    println!("\n=== DETAILED COMPARISON: {} vs {} ===", name1, name2);
    println!("Length: {} vs {}", arr1.len(), arr2.len());
    
    let min_len = std::cmp::min(arr1.len(), arr2.len());
    let mut differences = 0;
    let mut max_diff: f64 = 0.0;
    let mut total_diff: f64 = 0.0;
    
    for i in 0..min_len {
        let diff = (arr1[i] - arr2[i]).abs();
        if diff > 1e-15 {
            differences += 1;
            if differences <= 10 {
                println!("  Diff at [{}]: {:.15} vs {:.15} (diff: {:.2e})", i, arr1[i], arr2[i], diff);
            }
        }
        max_diff = max_diff.max(diff);
        total_diff += diff;
    }
    
    if differences == 0 {
        println!("✅ Arrays are IDENTICAL (within 1e-15 precision)");
    } else {
        println!("❌ Found {} differences out of {} elements", differences, min_len);
        println!("   Max difference: {:.2e}", max_diff);
        println!("   Average difference: {:.2e}", total_diff / min_len as f64);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== FFI CROSS-VALIDATION DEBUG TEST ===");
    
    // Use same seed as C implementation
    unsafe { srand(42) };
    
    let config = SpotConfig::default();
    let mut detector = SpotDetector::new(config)?;
    
    // Generate and collect training data
    let mut training_data = Vec::with_capacity(20000);
    for _ in 0..20000 {
        training_data.push(c_rand());
    }
    
    // Fit the model
    detector.fit(&training_data)?;
    println!("Model fitted. Initial threshold T = {}", detector.excess_threshold());
    
    // Process samples until we reach the target step
    let target_step = 100000;
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
    
    println!("\n=== FFI RESULTS AT STEP {} ===", step_count);
    let z_ffi = detector.anomaly_threshold();
    let t_ffi = detector.excess_threshold();
    let (gamma_ffi, sigma_ffi) = detector.get_gpd_parameters();
    let excess_values_ffi = detector.get_excess_values();
    let (e_ffi, e2_ffi, min_ffi, max_ffi, len_ffi) = detector.get_peaks_stats().unwrap();
    
    println!("Z={:.9} T={:.9} Gamma={:.9} Sigma={:.9}", z_ffi, t_ffi, gamma_ffi, sigma_ffi);
    println!("Excess count: {}, Normal count: {}, Anomaly count: {}", excess_count, normal_count, anomaly_count);
    println!("Excess buffer: {} values", len_ffi);
    println!("Peaks stats: e={:.9}, e2={:.9}, min={:.9}, max={:.9}", e_ffi, e2_ffi, min_ffi, max_ffi);
    
    // Save FFI excess values 
    let temp_dir = "/tmp";
    let ffi_excess_file = format!("{}/ffi_excess_values.txt", temp_dir);
    save_excess_to_file(&excess_values_ffi, &ffi_excess_file)?;
    println!("Saved {} excess values to {}", excess_values_ffi.len(), ffi_excess_file);
    
    // Sort FFI excess values
    let mut sorted_ffi = excess_values_ffi.clone();
    sorted_ffi.sort_by(|a, b| a.partial_cmp(b).unwrap());
    
    // Calculate FFI statistics manually
    let sum_ffi: f64 = excess_values_ffi.iter().sum();
    let sum_squares_ffi: f64 = excess_values_ffi.iter().map(|x| x * x).sum();
    let n_ffi = excess_values_ffi.len() as f64;
    let mean_calc_ffi = sum_ffi / n_ffi;
    let variance_calc_ffi = (sum_squares_ffi / n_ffi) - (mean_calc_ffi * mean_calc_ffi);
    
    println!("\nFFI Statistical verification:");
    println!("  Count: {}", excess_values_ffi.len());
    println!("  Sum: {:.15}", sum_ffi);
    println!("  Sum of squares: {:.15}", sum_squares_ffi);
    println!("  Mean (calculated): {:.15}", mean_calc_ffi);
    println!("  Variance (calculated): {:.15}", variance_calc_ffi);
    println!("  Min: {:.15}", sorted_ffi[0]);
    println!("  Max: {:.15}", sorted_ffi[sorted_ffi.len() - 1]);
    
    // Try to load the Rust excess values
    let rust_excess_file = format!("{}/rust_excess_values.txt", temp_dir);
    match load_excess_from_file(&rust_excess_file) {
        Ok(excess_values_rust) => {
            println!("\n=== LOADED RUST EXCESS VALUES ===");
            println!("Loaded {} values from {}", excess_values_rust.len(), rust_excess_file);
            
            // Sort Rust values for comparison
            let mut sorted_rust = excess_values_rust.clone();
            sorted_rust.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            // Compare the sorted arrays point by point
            compare_arrays_detailed(&sorted_ffi, &sorted_rust, "FFI", "Rust");
            
            // Calculate Rust statistics  
            let sum_rust: f64 = excess_values_rust.iter().sum();
            let sum_squares_rust: f64 = excess_values_rust.iter().map(|x| x * x).sum();
            let n_rust = excess_values_rust.len() as f64;
            let mean_calc_rust = sum_rust / n_rust;
            let variance_calc_rust = (sum_squares_rust / n_rust) - (mean_calc_rust * mean_calc_rust);
            
            println!("\nRust Statistical verification:");
            println!("  Count: {}", excess_values_rust.len());
            println!("  Sum: {:.15}", sum_rust);
            println!("  Sum of squares: {:.15}", sum_squares_rust);
            println!("  Mean (calculated): {:.15}", mean_calc_rust);
            println!("  Variance (calculated): {:.15}", variance_calc_rust);
            println!("  Min: {:.15}", sorted_rust[0]);
            println!("  Max: {:.15}", sorted_rust[sorted_rust.len() - 1]);
            
            // Compare statistics
            println!("\n=== STATISTICAL COMPARISON ===");
            println!("Count: {} vs {} (diff: {})", excess_values_ffi.len(), excess_values_rust.len(), 
                     excess_values_ffi.len() as i64 - excess_values_rust.len() as i64);
            println!("Sum: {:.2e} vs {:.2e} (diff: {:.2e})", sum_ffi, sum_rust, (sum_ffi - sum_rust).abs());
            println!("Mean: {:.2e} vs {:.2e} (diff: {:.2e})", mean_calc_ffi, mean_calc_rust, (mean_calc_ffi - mean_calc_rust).abs());
            println!("Variance: {:.2e} vs {:.2e} (diff: {:.2e})", variance_calc_ffi, variance_calc_rust, (variance_calc_ffi - variance_calc_rust).abs());
            
            if excess_values_ffi.len() == excess_values_rust.len() &&
               (sum_ffi - sum_rust).abs() < 1e-12 &&
               (mean_calc_ffi - mean_calc_rust).abs() < 1e-12 {
                println!("\n✅ INPUT DATA IS IDENTICAL BETWEEN IMPLEMENTATIONS");
                println!("The difference must be in the Grimshaw estimator function implementation.");
                
                // Now test: feed Rust excess values to FFI Grimshaw estimator
                println!("\n=== TESTING: FFI GRIMSHAW WITH RUST INPUTS ===");
                let (gamma_ffi_with_rust, sigma_ffi_with_rust, ll_ffi_with_rust) = 
                    detector.call_grimshaw_estimator_with_data(&excess_values_rust);
                    
                println!("FFI Grimshaw with Rust inputs:");
                println!("  Gamma={:.9} Sigma={:.9} LogLikelihood={:.9}", 
                         gamma_ffi_with_rust, sigma_ffi_with_rust, ll_ffi_with_rust);
                         
                println!("FFI Grimshaw with FFI inputs:");
                println!("  Gamma={:.9} Sigma={:.9}", gamma_ffi, sigma_ffi);
                
                println!("Comparison:");
                println!("  Gamma diff: {:.2e}", (gamma_ffi - gamma_ffi_with_rust).abs());
                println!("  Sigma diff: {:.2e}", (sigma_ffi - sigma_ffi_with_rust).abs());
                
                if (gamma_ffi - gamma_ffi_with_rust).abs() < 1e-12 && 
                   (sigma_ffi - sigma_ffi_with_rust).abs() < 1e-12 {
                    println!("\n✅ FFI Grimshaw produces IDENTICAL results with both input sets");
                    println!("This confirms the problem is in the pure Rust Grimshaw implementation!");
                } else {
                    println!("\n❌ FFI Grimshaw produces DIFFERENT results with different inputs");
                    println!("This suggests there may be subtle input data differences or state issues.");
                }
            } else {
                println!("\n❌ INPUT DATA IS DIFFERENT BETWEEN IMPLEMENTATIONS");
                println!("The difference is in the data generation/processing pipeline, not Grimshaw.");
            }
        }
        Err(e) => {
            println!("\n⚠️  Could not load Rust excess values: {}", e);
            println!("Run the Rust debug_cross_validate example first to generate the data file.");
        }
    }
    
    Ok(())
}