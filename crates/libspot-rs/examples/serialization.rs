//! Example demonstrating model serialization and deserialization
//!
//! This example shows how to:
//! 1. Train a SPOT detector
//! 2. Serialize (save) the trained model to JSON
//! 3. Deserialize (load) the model from JSON
//! 4. Continue using the loaded model for anomaly detection
//!
//! Run with: cargo run --example serialization

#![cfg(feature = "serde")]

use libspot_rs::{SpotConfig, SpotDetector, SpotStatus};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SPOT Model Serialization Example ===\n");

    // Step 1: Create and train a SPOT detector
    println!("1. Creating and training SPOT detector...");
    let config = SpotConfig {
        q: 0.001,
        level: 0.98,
        max_excess: 100,
        ..SpotConfig::default()
    };
    let mut detector = SpotDetector::new(config)?;

    // Generate training data (normal distribution simulation)
    let training_data: Vec<f64> = (0..1000)
        .map(|i| 10.0 + (i as f64 * 0.1).sin() * 2.0)
        .collect();
    detector.fit(&training_data)?;

    println!("   Training complete!");
    println!(
        "   - Anomaly threshold: {:.4}",
        detector.anomaly_threshold()
    );
    println!("   - Excess threshold: {:.4}", detector.excess_threshold());
    println!("   - Data points seen: {}", detector.n());

    // Step 2: Serialize the trained model to JSON
    println!("\n2. Serializing model to JSON...");
    let json = serde_json::to_string_pretty(&detector)?;
    println!("   Model serialized ({} bytes)", json.len());

    // In a real application, you would save to a file:
    // std::fs::write("model.json", &json)?;
    // println!("   Saved to model.json");

    // Step 3: Deserialize the model from JSON
    println!("\n3. Deserializing model from JSON...");
    // In a real application, you would load from a file:
    // let json = std::fs::read_to_string("model.json")?;
    let loaded_detector: SpotDetector = serde_json::from_str(&json)?;
    println!("   Model loaded successfully!");
    println!(
        "   - Anomaly threshold: {:.4}",
        loaded_detector.anomaly_threshold()
    );
    println!(
        "   - Excess threshold: {:.4}",
        loaded_detector.excess_threshold()
    );

    // Verify the loaded model has the same state
    assert_eq!(detector.n(), loaded_detector.n());
    assert_eq!(detector.nt(), loaded_detector.nt());
    println!("   ✓ Model state verified!");

    // Step 4: Use the loaded model for anomaly detection
    println!("\n4. Using loaded model for anomaly detection...");
    let mut loaded_detector = loaded_detector; // Make mutable for step()

    let test_values = [
        (10.5, "normal value"),
        (11.0, "normal value"),
        (50.0, "anomalous spike"),
        (10.2, "normal value"),
        (100.0, "extreme anomaly"),
    ];

    for (value, description) in test_values {
        let status = loaded_detector.step(value)?;
        let status_str = match status {
            SpotStatus::Normal => "Normal",
            SpotStatus::Excess => "Excess",
            SpotStatus::Anomaly => "ANOMALY",
        };
        println!(
            "   Value: {:6.1} ({}) -> {}",
            value, description, status_str
        );
    }

    // Demonstrate saving after processing more data
    println!("\n5. Re-serializing after processing more data...");
    let updated_json = serde_json::to_string(&loaded_detector)?;
    println!("   Updated model serialized ({} bytes)", updated_json.len());
    println!("   - Data points now seen: {}", loaded_detector.n());

    println!("\n=== Example Complete ===");
    println!("\nKey points:");
    println!("- Models can be serialized at any point after training");
    println!("- Loaded models retain all state and thresholds");
    println!("- Models can be re-serialized after processing more data");
    println!("- Use serde_json for JSON, or other serde formats (bincode, etc.)");

    Ok(())
}
