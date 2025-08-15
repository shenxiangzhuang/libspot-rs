//! Compare exact Grimshaw computation at a specific point

use libspot::{Spot, SpotConfig, SpotStatus};

extern "C" {
    fn srand(seed: u32);
    fn rand() -> i32;
}

fn c_rand() -> f64 {
    unsafe { rand() as f64 / (i32::MAX as f64 + 1.0) }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PURE RUST SINGLE STEP GRIMSHAW DEBUG ===");
    
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
    
    // Process until a specific point and capture detailed state
    let mut step_count = 0;
    let target_step = 100000; // Point where we know there's no divergence yet
    
    for _ in 0..target_step {
        let value = c_rand();
        let _status = detector.step(value)?;
        step_count += 1;
    }
    
    // Now capture the exact state before next Grimshaw call
    println!("\n=== CAPTURING STATE AT STEP {} ===", step_count);
    let excess_values = detector.get_excess_values();
    let (gamma, sigma) = detector.get_gpd_parameters();
    
    println!("Current GPD: gamma={:.15}, sigma={:.15}", gamma, sigma);
    println!("Excess buffer size: {}", excess_values.len());
    
    if excess_values.len() > 0 {
        println!("Excess values for Grimshaw input:");
        for (i, val) in excess_values.iter().enumerate() {
            if i < 10 || i >= excess_values.len() - 10 {
                println!("  [{}] = {:.15}", i, val);
            } else if i == 10 {
                println!("  ... ({} more values) ...", excess_values.len() - 20);
            }
        }
        
        let min = excess_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let max = excess_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let mean = excess_values.iter().sum::<f64>() / excess_values.len() as f64;
        let sum = excess_values.iter().sum::<f64>();
        
        println!("Statistics: min={:.15}, max={:.15}, mean={:.15}, sum={:.15}", min, max, mean, sum);
    }
    
    // Trigger one more step to see Grimshaw in detail
    std::env::set_var("SPOT_DEBUG_GRIMSHAW", "1");
    let value = c_rand();
    let _status = detector.step(value)?;
    step_count += 1;
    
    let (new_gamma, new_sigma) = detector.get_gpd_parameters();
    println!("After step {}: gamma={:.15}, sigma={:.15}", step_count, new_gamma, new_sigma);
    
    Ok(())
}