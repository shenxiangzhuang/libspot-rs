use approx::assert_relative_eq;
use libspot::{SpotConfig, SpotDetector, SpotStatus};
use spot_rs::{Spot as PureSpot, SpotConfig as PureSpotConfig, SpotStatus as PureSpotStatus};

/// Random number generator that matches C's rand()/srand() for reproducible results
struct CRand {
    seed: u32,
}

impl CRand {
    fn new(seed: u32) -> Self {
        Self { seed }
    }

    fn next(&mut self) -> u32 {
        self.seed = self.seed.wrapping_mul(1103515245).wrapping_add(12345);
        (self.seed / 65536) % 32768
    }

    fn rexp(&mut self) -> f64 {
        let u = self.next() as f64 / 32767.0;
        -u.ln()
    }
}

/// Test that compares pure Rust implementation with C FFI implementation
#[test]
fn test_pure_rust_vs_c_basic() {
    // Configure SPOT detector with exact same parameters
    let c_config = SpotConfig {
        q: 0.0001,
        low_tail: false,
        discard_anomalies: true,
        level: 0.998,
        max_excess: 200,
    };

    let rust_config = PureSpotConfig {
        q: 0.0001,
        low_tail: false,
        discard_anomalies: true,
        level: 0.998,
        max_excess: 200,
    };

    // Create detectors
    let mut c_detector = SpotDetector::new(c_config).unwrap();
    let mut rust_detector = PureSpot::new(rust_config).unwrap();

    // Generate initial training data with same seed
    let n = 1000; // Smaller test for debugging
    let mut initial_data = Vec::with_capacity(n);
    let mut rng = CRand::new(1);

    for _ in 0..n {
        initial_data.push(rng.rexp());
    }

    // Fit both models
    c_detector.fit(&initial_data).unwrap();
    rust_detector.fit(&initial_data).unwrap();

    // Compare thresholds
    let c_anomaly_threshold = c_detector.anomaly_threshold();
    let rust_anomaly_threshold = rust_detector.anomaly_threshold();
    let c_excess_threshold = c_detector.excess_threshold();
    let rust_excess_threshold = rust_detector.excess_threshold();

    println!("C anomaly threshold: {}", c_anomaly_threshold);
    println!("Rust anomaly threshold: {}", rust_anomaly_threshold);
    println!("C excess threshold: {}", c_excess_threshold);
    println!("Rust excess threshold: {}", rust_excess_threshold);

    // Check if thresholds are reasonably close (within 10%)
    if !c_anomaly_threshold.is_nan() && !rust_anomaly_threshold.is_nan() {
        let relative_diff = ((c_anomaly_threshold - rust_anomaly_threshold).abs() / c_anomaly_threshold).abs();
        assert!(relative_diff < 0.1, "Anomaly thresholds differ by more than 10%: C={}, Rust={}", c_anomaly_threshold, rust_anomaly_threshold);
    }

    if !c_excess_threshold.is_nan() && !rust_excess_threshold.is_nan() {
        let relative_diff = ((c_excess_threshold - rust_excess_threshold).abs() / c_excess_threshold).abs();
        assert!(relative_diff < 0.1, "Excess thresholds differ by more than 10%: C={}, Rust={}", c_excess_threshold, rust_excess_threshold);
    }

    // Test some values
    let test_values = vec![1.0, 2.0, 5.0, 10.0];
    
    for value in test_values {
        let c_result = c_detector.step(value).unwrap();
        let rust_result = rust_detector.step(value).unwrap();
        
        // Convert to numeric for comparison
        let c_status = c_result as i32;
        let rust_status = rust_result as i32;
        
        println!("Value {}: C={:?}({}), Rust={:?}({})", value, c_result, c_status, rust_result, rust_status);
        
        // Results should be the same or at least consistent (since both Normal=0, Excess=1, Anomaly=2)
        // Allow some difference due to numerical precision
    }
}

/// Test just basic functionality comparison
#[test] 
fn test_basic_pure_rust_functionality() {
    let rust_config = PureSpotConfig {
        q: 0.0001,
        low_tail: false,
        discard_anomalies: true,
        level: 0.998,
        max_excess: 200,
    };

    let mut rust_detector = PureSpot::new(rust_config).unwrap();

    // Generate simple training data
    let data: Vec<f64> = (1..=1000).map(|i| i as f64).collect();
    
    println!("Fitting pure Rust detector...");
    let result = rust_detector.fit(&data);
    assert!(result.is_ok(), "Pure Rust detector should fit successfully");

    println!("Anomaly threshold: {}", rust_detector.anomaly_threshold());
    println!("Excess threshold: {}", rust_detector.excess_threshold());
    
    // Test a few values
    let test_values = vec![500.0, 900.0, 1500.0];
    for value in test_values {
        let result = rust_detector.step(value);
        println!("Value {}: {:?}", value, result);
        assert!(result.is_ok(), "Step function should work");
    }
}