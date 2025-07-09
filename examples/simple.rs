//! Simple usage example of the libspot library
//!
//! This example shows the minimal code needed to use SPOT for anomaly detection.

use libspot::{version, SpotConfig, SpotDetector, SpotStatus};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("SPOT Anomaly Detection - Simple Example");
    println!("Library version: {}", version());

    // Create a SPOT detector with default configuration
    let config = SpotConfig::default();
    let mut detector = SpotDetector::new(config)?;

    // Generate some training data (normal data)
    let training_data: Vec<f64> = (0..1000)
        .map(|_| rand::random::<f64>() * 10.0) // Random values 0-10
        .collect();

    // Fit the model
    detector.fit(&training_data)?;
    println!("Model fitted with {} training samples", training_data.len());

    // Test with some data points
    let test_values = vec![5.0, 8.0, 15.0, 25.0, 100.0]; // Last two should be anomalies

    for value in test_values {
        match detector.step(value)? {
            SpotStatus::Normal => println!("Value {value:.1}: NORMAL"),
            SpotStatus::Excess => println!("Value {value:.1}: EXCESS (in tail)"),
            SpotStatus::Anomaly => println!("Value {value:.1}: ANOMALY! 🚨"),
        }
    }

    println!("Current thresholds:");
    println!("  Excess threshold: {:.3}", detector.excess_threshold());
    println!("  Anomaly threshold: {:.3}", detector.anomaly_threshold());

    Ok(())
}
