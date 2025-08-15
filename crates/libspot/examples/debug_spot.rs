use libspot::{p2_quantile, Spot, SpotConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate exponential random data like in the example
    let mut rng_state = 1u32;
    let mut generate_exp = || {
        rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
        let uniform = (rng_state / 65536) % 32768;
        let uniform_float = uniform as f64 / 32767.0;
        -uniform_float.ln()
    };
    
    let data: Vec<f64> = (0..20000).map(|_| generate_exp()).collect();
    
    println!("Data length: {}", data.len());
    println!("Data range: {} to {}", data.iter().fold(f64::INFINITY, |a, &b| a.min(b)), data.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b)));
    
    // Try direct P2 quantile calculation
    let quantile = p2_quantile(0.998, &data);
    println!("Direct P2 quantile (0.998): {}", quantile);
    
    if quantile.is_nan() {
        println!("Quantile is NaN - let's debug");
        
        // Try with smaller level
        let smaller_quantile = p2_quantile(0.95, &data);
        println!("P2 quantile (0.95): {}", smaller_quantile);
        
        // Try with very small data
        let small_data: Vec<f64> = data[0..10].to_vec();
        let small_quantile = p2_quantile(0.998, &small_data);
        println!("P2 quantile on {} elements: {}", small_data.len(), small_quantile);
    } else {
        let config = SpotConfig {
            q: 0.0001,
            low_tail: false,
            discard_anomalies: true,
            level: 0.998,
            max_excess: 200,
        };
        
        let mut detector = Spot::new(config)?;
        println!("SPOT detector created");
        
        match detector.fit(&data) {
            Ok(_) => {
                println!("Fit successful!");
                println!("Excess threshold: {}", detector.excess_threshold());
                println!("Anomaly threshold: {}", detector.anomaly_threshold());
            }
            Err(e) => {
                println!("Fit failed: {:?}", e);
            }
        }
    }
    
    Ok(())
}