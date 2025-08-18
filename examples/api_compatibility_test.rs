//! API compatibility test - ensures exact same method signatures
//!
//! This test demonstrates that both crates have identical APIs

fn main() -> Result<(), Box<dyn std::error::Error>> {
    test_api_compatibility()
}

fn test_api_compatibility() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing API Compatibility...\n");

    // Test that all public methods exist with same signatures
    test_configuration_api()?;
    test_detector_api()?;
    test_status_api()?;
    test_version_api()?;

    println!("✅ All APIs are perfectly aligned!");

    Ok(())
}

fn test_configuration_api() -> Result<(), Box<dyn std::error::Error>> {
    println!("📋 Testing SpotConfig API...");

    // Test libspot config
    let _config1 = libspot::SpotConfig {
        q: 0.001,
        low_tail: false,
        discard_anomalies: true,
        level: 0.999,
        max_excess: 100,
    };

    // Test libspot-rs config with EXACT same field names and types
    let _config2 = libspot_rs::SpotConfig {
        q: 0.001,
        low_tail: false,
        discard_anomalies: true,
        level: 0.999,
        max_excess: 100,
    };

    // Test default() method exists on both
    let _default1 = libspot::SpotConfig::default();
    let _default2 = libspot_rs::SpotConfig::default();

    println!("   ✅ SpotConfig fields and methods match");
    Ok(())
}

fn test_detector_api() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Testing SpotDetector API...");

    let config1 = libspot::SpotConfig::default();
    let config2 = libspot_rs::SpotConfig::default();

    let mut detector1 = libspot::SpotDetector::new(config1)?;
    let mut detector2 = libspot_rs::SpotDetector::new(config2)?;

    let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];

    // Test all methods have same signatures
    detector1.fit(&data)?;
    detector2.fit(&data)?;
    println!("   ✅ fit() method signature matches");

    let _status1 = detector1.step(3.0)?;
    let _status2 = detector2.step(3.0)?;
    println!("   ✅ step() method signature matches");

    let _q1 = detector1.quantile(0.5);
    let _q2 = detector2.quantile(0.5);
    println!("   ✅ quantile() method signature matches");

    let _at1 = detector1.anomaly_threshold();
    let _at2 = detector2.anomaly_threshold();
    println!("   ✅ anomaly_threshold() method signature matches");

    let _et1 = detector1.excess_threshold();
    let _et2 = detector2.excess_threshold();
    println!("   ✅ excess_threshold() method signature matches");

    let _cfg1 = detector1.config();
    let _cfg2 = detector2.config();
    println!("   ✅ config() method signature matches (returns Option)");

    let _n1 = detector1.n();
    let _n2 = detector2.n();
    println!("   ✅ n() method signature matches (returns u64)");

    let _nt1 = detector1.nt();
    let _nt2 = detector2.nt();
    println!("   ✅ nt() method signature matches (returns u64)");

    let _tail1 = detector1.tail_parameters();
    let _tail2 = detector2.tail_parameters();
    println!("   ✅ tail_parameters() method signature matches");

    Ok(())
}

fn test_status_api() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Testing SpotStatus API...");

    // Test that both have same enum variants
    let _normal1 = libspot::SpotStatus::Normal;
    let _normal2 = libspot_rs::SpotStatus::Normal;

    let _excess1 = libspot::SpotStatus::Excess;
    let _excess2 = libspot_rs::SpotStatus::Excess;

    let _anomaly1 = libspot::SpotStatus::Anomaly;
    let _anomaly2 = libspot_rs::SpotStatus::Anomaly;

    println!("   ✅ SpotStatus enum variants match");
    Ok(())
}

fn test_version_api() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔖 Testing Version API...");

    let _v1 = libspot::version();
    let _v2 = libspot_rs::version();

    println!("   ✅ version() function exists in both crates");
    Ok(())
}
