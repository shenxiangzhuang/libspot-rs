//! Basic example demonstrating the pure Rust libspot library
//!
//! This example demonstrates the pure Rust implementation with deterministic data.

use libspot::{Spot, SpotConfig, SpotStatus};
use std::time::Instant;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Pure Rust SPOT Basic Example");
    
    // Create configuration
    let config = SpotConfig {
        q: 0.0001,
        low_tail: false,
        discard_anomalies: true,
        level: 0.998,
        max_excess: 200,
    };
    
    let mut detector = Spot::new(config)?;
    
    // Generate deterministic training data (sine wave with some variance)
    let training_data: Vec<f64> = (0..1000)
        .map(|i| {
            let x = i as f64 * 0.01;
            1.0 + 0.5 * (x * std::f64::consts::PI).sin() + 0.1 * ((x * 5.0).sin())
        })
        .collect();
    
    println!("Training SPOT detector with {} samples...", training_data.len());
    let start = Instant::now();
    detector.fit(&training_data)?;
    let fit_time = start.elapsed();
    
    println!("Training completed in {:.2?}", fit_time);
    println!("Excess threshold: {:.6}", detector.excess_threshold());
    println!("Anomaly threshold: {:.6}", detector.anomaly_threshold());
    
    // Test with various values including anomalies
    let test_values = vec![1.0, 1.5, 2.0, 3.0, 5.0, 10.0];
    
    println!("\nTesting values:");
    let start = Instant::now();
    for value in test_values {
        match detector.step(value)? {
            SpotStatus::Normal => println!("Value {:.1}: Normal", value),
            SpotStatus::Excess => println!("Value {:.1}: Excess (in tail)", value),
            SpotStatus::Anomaly => println!("Value {:.1}: ANOMALY!", value),
        }
    }
    let test_time = start.elapsed();
    
    println!("\nTesting completed in {:.2?}", test_time);
    println!("Final excess threshold: {:.6}", detector.excess_threshold());
    println!("Final anomaly threshold: {:.6}", detector.anomaly_threshold());
    
    Ok(())
}