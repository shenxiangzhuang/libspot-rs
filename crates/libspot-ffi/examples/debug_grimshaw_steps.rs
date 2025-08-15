//! Step-by-step Grimshaw debug comparison

use libspot_ffi::{SpotDetector, SpotConfig, SpotStatus};

extern "C" {
    fn srand(seed: u32);
    fn rand() -> i32;
}

fn c_rand() -> f64 {
    unsafe { rand() as f64 / (i32::MAX as f64 + 1.0) }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== FFI GRIMSHAW STEP DEBUG ===");
    
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
    
    // Run until we see divergence
    let mut step_count = 0;
    
    // Process until a specific point where we know divergence starts
    for _ in 0..500000 {
        let value = c_rand();
        let status = detector.step(value)?;
        step_count += 1;
        
        // Log detailed Grimshaw behavior at certain points
        if step_count == 100000 || step_count == 300000 || step_count == 500000 {
            let z = detector.anomaly_threshold();
            let t = detector.excess_threshold();
            let (gamma, sigma) = detector.get_gpd_parameters();
            let excess_values = detector.get_excess_values();
            
            println!("\n=== DETAILED GRIMSHAW ANALYSIS AT STEP {} ===", step_count);
            println!("Final result: Z={:.15} T={:.15} Gamma={:.15} Sigma={:.15}", z, t, gamma, sigma);
            
            if excess_values.len() > 0 {
                println!("Excess buffer size: {}", excess_values.len());
                println!("Min: {:.15}, Max: {:.15}, Mean: {:.15}", 
                        excess_values.iter().cloned().fold(f64::INFINITY, f64::min),
                        excess_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max),
                        excess_values.iter().sum::<f64>() / excess_values.len() as f64);
            }
        }
        
        match status {
            SpotStatus::Anomaly => {},
            SpotStatus::Excess => {},
            SpotStatus::Normal => {},
        }
    }
    
    println!("\n=== FINAL GRIMSHAW DEBUG RESULTS ===");
    println!("Steps processed: {}", step_count);
    println!("Z={:.15} T={:.15}", detector.anomaly_threshold(), detector.excess_threshold());
    
    Ok(())
}