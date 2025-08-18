//! Unified API Demo - showing exact interchangeability
//!
//! This example demonstrates that both libspot and libspot-rs have
//! exactly the same API and can be used interchangeably by just
//! changing the crate import.

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Unified API Demo ===");
    println!("Both crates now use EXACTLY the same types and method names!\n");

    // Test with synthetic data
    let training_data: Vec<f64> = (0..1000).map(|i| (i as f64 * 0.01).sin()).collect();
    let test_values = vec![0.1, 0.5, 2.0, 10.0]; // 10.0 should be anomaly

    demo_with_libspot(&training_data, &test_values)?;
    println!();
    demo_with_libspot_rs(&training_data, &test_values)?;

    println!("\n✅ Both implementations work identically!");
    println!("💡 To switch implementations, just change the crate import!");

    Ok(())
}

/// Using libspot (C FFI) - note the identical API
fn demo_with_libspot(
    training_data: &[f64],
    test_values: &[f64],
) -> Result<(), Box<dyn std::error::Error>> {
    // The EXACT same API as libspot-rs!
    use libspot::{SpotConfig, SpotDetector, SpotStatus};

    println!("🔧 libspot (C FFI):");

    let config = SpotConfig::default();
    let mut detector = SpotDetector::new(config)?;
    detector.fit(training_data)?;

    println!("   Anomaly threshold: {:.3}", detector.anomaly_threshold());

    for &value in test_values {
        let status = detector.step(value)?;
        match status {
            SpotStatus::Normal => println!("   {:.1} → Normal", value),
            SpotStatus::Excess => println!("   {:.1} → Excess", value),
            SpotStatus::Anomaly => println!("   {:.1} → ⚠️  ANOMALY", value),
        }
    }

    println!("   Version: {}", libspot::version());

    Ok(())
}

/// Using libspot-rs (Pure Rust) - note the IDENTICAL API
fn demo_with_libspot_rs(
    training_data: &[f64],
    test_values: &[f64],
) -> Result<(), Box<dyn std::error::Error>> {
    // The EXACT same API as libspot!
    use libspot_rs::{SpotConfig, SpotDetector, SpotStatus};

    println!("🦀 libspot-rs (Pure Rust):");

    let config = SpotConfig::default();
    let mut detector = SpotDetector::new(config)?; // Same method name
    detector.fit(training_data)?; // Same method name

    println!("   Anomaly threshold: {:.3}", detector.anomaly_threshold()); // Same method

    for &value in test_values {
        let status = detector.step(value)?; // Same method signature
        match status {
            SpotStatus::Normal => println!("   {:.1} → Normal", value), // Same enum variants
            SpotStatus::Excess => println!("   {:.1} → Excess", value),
            SpotStatus::Anomaly => println!("   {:.1} → ⚠️  ANOMALY", value),
        }
    }

    println!("   Version: {}", libspot_rs::version());

    Ok(())
}
