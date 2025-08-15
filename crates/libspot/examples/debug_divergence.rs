//! Debug at divergence point

use libspot::{Spot, SpotConfig, SpotStatus};

extern "C" {
    fn srand(seed: u32);
    fn rand() -> i32;
}

fn c_rand() -> f64 {
    unsafe { rand() as f64 / (i32::MAX as f64 + 1.0) }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PURE RUST DIVERGENCE DEBUG ===");
    
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
    
    // Test around the divergence point
    let target_steps = [100000, 200000, 300000, 500000, 750000, 1000000];
    let mut step_count = 0;
    let mut anomaly_count = 0;
    let mut excess_count = 0;
    let mut normal_count = 0;
    
    println!("Processing 1M samples...");
    for _ in 0..1000000 {
        let value = c_rand();
        let status = detector.step(value)?;
        step_count += 1;
        
        match status {
            SpotStatus::Anomaly => {
                anomaly_count += 1;
            },
            SpotStatus::Excess => {
                excess_count += 1;
            },
            SpotStatus::Normal => {
                normal_count += 1;
            },
        }
        
        if target_steps.contains(&step_count) {
            let z = detector.anomaly_threshold();
            let t = detector.excess_threshold();
            let (gamma, sigma) = detector.get_gpd_parameters();
            let excess_values = detector.get_excess_values();
            
            println!("\n=== STEP {} ===", step_count);
            println!("Z={:.6} T={:.6} Gamma={:.6} Sigma={:.6}", z, t, gamma, sigma);
            println!("Excess count: {}, Normal count: {}, Anomaly count: {}", 
                    excess_count, normal_count, anomaly_count);
            
            if excess_values.len() > 0 {
                println!("Total excess values in buffer: {}", excess_values.len());
                
                let sum: f64 = excess_values.iter().sum();
                let mean = sum / excess_values.len() as f64;
                let variance = excess_values.iter()
                    .map(|x| (x - mean).powi(2))
                    .sum::<f64>() / excess_values.len() as f64;
                
                println!("Excess statistics - Mean: {:.6}, Variance: {:.6}, Sum: {:.6}", 
                        mean, variance, sum);
                        
                // Print min/max for better understanding
                let min = excess_values.iter().cloned().fold(f64::INFINITY, f64::min);
                let max = excess_values.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                println!("Min: {:.6}, Max: {:.6}", min, max);
            }
        }
    }
    
    println!("\n=== FINAL RESULTS ===");
    println!("ANOMALY={} EXCESS={} NORMAL={}", anomaly_count, excess_count, normal_count);
    println!("Z={:.6} T={:.6}", detector.anomaly_threshold(), detector.excess_threshold());
    
    Ok(())
}