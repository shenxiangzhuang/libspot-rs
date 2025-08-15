//! Demonstration of pure Rust SPOT implementation
//!
//! This example shows how to use the pure Rust SPOT implementation
//! for time series anomaly detection.

use libspot::{Spot, SpotConfig, SpotStatus};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SPOT Anomaly Detection - Pure Rust Implementation");
    
    // Create a SPOT detector with default configuration
    let config = SpotConfig::default();
    let mut detector = Spot::new(config)?;

    // Generate some training data (exponential distribution)
    let training_data: Vec<f64> = (0..1000)
        .map(|i| {
            let u = (i as f64 + 1.0) / 1001.0; // Avoid 0 and 1
            -u.ln() // Exponential distribution
        })
        .collect();

    // Fit the model
    detector.fit(&training_data)?;
    println!("Model fitted with {} training samples", training_data.len());
    println!("Excess threshold: {:.3}", detector.excess_threshold());
    println!("Anomaly threshold: {:.3}", detector.anomaly_threshold());

    // Test with some data points
    let test_values = vec![0.5, 1.0, 2.0, 5.0, 10.0, 20.0];

    println!("\nTesting values:");
    for value in test_values {
        match detector.step(value)? {
            SpotStatus::Normal => println!("Value {:.1}: NORMAL", value),
            SpotStatus::Excess => println!("Value {:.1}: EXCESS (in tail)", value),
            SpotStatus::Anomaly => println!("Value {:.1}: ANOMALY! 🚨", value),
        }
    }

    println!("\nFinal thresholds:");
    println!("  Excess threshold: {:.3}", detector.excess_threshold());
    println!("  Anomaly threshold: {:.3}", detector.anomaly_threshold());

    Ok(())
}