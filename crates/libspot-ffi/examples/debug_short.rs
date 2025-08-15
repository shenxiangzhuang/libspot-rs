use libspot_ffi::{SpotDetector, SpotConfig, SpotStatus};

extern "C" {
    fn srand(seed: u32);
    fn rand() -> i32;
}

fn c_rand() -> f64 {
    unsafe { rand() as f64 / (i32::MAX as f64 + 1.0) }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== FFI SHORT DEBUG ===");
    
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
    
    // Test for just 100k steps
    let target_steps = [50000, 100000];
    let mut step_count = 0;
    let mut anomaly_count = 0;
    let mut excess_count = 0;
    let mut normal_count = 0;
    
    for _ in 0..100000 {
        let value = c_rand();
        let status = detector.step(value)?;
        step_count += 1;
        
        match status {
            SpotStatus::Anomaly => {
                anomaly_count += 1;
                if step_count <= 1000 { print!("!"); }
            },
            SpotStatus::Excess => {
                excess_count += 1;
                if step_count <= 1000 { print!("."); }
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
                println!("First 10 excess values: {:?}", 
                        &excess_values[..std::cmp::min(10, excess_values.len())]);
                if excess_values.len() > 10 {
                    println!("Last 10 excess values: {:?}", 
                            &excess_values[excess_values.len().saturating_sub(10)..]);
                }
                
                let sum: f64 = excess_values.iter().sum();
                let mean = sum / excess_values.len() as f64;
                let variance = excess_values.iter()
                    .map(|x| (x - mean).powi(2))
                    .sum::<f64>() / excess_values.len() as f64;
                
                println!("Excess statistics - Mean: {:.6}, Variance: {:.6}, Sum: {:.6}", 
                        mean, variance, sum);
            }
        }
    }
    
    println!("\n=== FINAL RESULTS ===");
    println!("ANOMALY={} EXCESS={} NORMAL={}", anomaly_count, excess_count, normal_count);
    println!("Z={:.6} T={:.6}", detector.anomaly_threshold(), detector.excess_threshold());
    
    let final_excess = detector.get_excess_values();
    let (final_gamma, final_sigma) = detector.get_gpd_parameters();
    println!("Final GPD: Gamma={:.6} Sigma={:.6}", final_gamma, final_sigma);
    println!("Final excess buffer size: {}", final_excess.len());
    
    Ok(())
}