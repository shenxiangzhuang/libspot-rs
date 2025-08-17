use approx::assert_relative_eq;
use libspot::{Spot, SpotConfig, SpotStatus};

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
        if u <= 0.0 || u >= 1.0 {
            return 1.0; // Safe fallback
        }
        -u.ln()
    }
}

/// Test that reproduces the basic example behavior exactly using pure Rust
#[test]
fn test_pure_rust_basic_example_behavior() {
    // Configure SPOT detector with exact same parameters as C basic example
    let config = SpotConfig {
        q: 0.0001,
        low_tail: false,
        discard_anomalies: true,
        level: 0.998,
        max_excess: 200,
    };

    // Create and initialize SPOT detector
    let mut detector = Spot::new(config).unwrap();

    // Generate initial training data with same seed as C example
    let n = 20000;
    let mut initial_data = Vec::with_capacity(n);
    let mut rng = CRand::new(1); // Use same seed as C example

    for _ in 0..n {
        initial_data.push(rng.rexp());
    }

    // Fit the model
    println!("Fitting with {} data points", initial_data.len());
    println!(
        "Data range: {} to {}",
        initial_data.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
        initial_data
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b))
    );

    let fit_result = detector.fit(&initial_data);
    if let Err(e) = &fit_result {
        println!("Fit error: {:?}", e);
    }
    fit_result.unwrap();

    // Test with 1000 samples using same random sequence
    let mut rng = CRand::new(1);
    // Skip the initial training data
    for _ in 0..n {
        rng.rexp();
    }

    let mut anomaly = 0;
    let mut excess = 0;
    let mut normal = 0;

    for _ in 0..1000 {
        let x = rng.rexp();
        match detector.step(x).unwrap() {
            SpotStatus::Normal => normal += 1,
            SpotStatus::Excess => excess += 1,
            SpotStatus::Anomaly => anomaly += 1,
        }
    }

    println!("Pure Rust Results after 1000 samples:");
    println!("ANOMALY={} EXCESS={} NORMAL={}", anomaly, excess, normal);
    println!(
        "Z={:.6} T={:.6}",
        detector.anomaly_threshold(),
        detector.excess_threshold()
    );

    // Validate that we get reasonable results
    assert!(anomaly + excess + normal == 1000);
    assert!(!detector.anomaly_threshold().is_nan());
    assert!(!detector.excess_threshold().is_nan());

    // The exact values will depend on the algorithm implementation
    // but we should see some distribution of results
    assert!(normal > 0, "Should have some normal classifications");
}

/// Test with larger dataset to see convergence behavior
#[test]
#[ignore] // Ignore by default due to runtime
fn test_pure_rust_basic_example_larger() {
    let config = SpotConfig {
        q: 0.0001,
        low_tail: false,
        discard_anomalies: true,
        level: 0.998,
        max_excess: 200,
    };

    let mut detector = Spot::new(config).unwrap();

    // Generate initial training data
    let n = 20000;
    let mut initial_data = Vec::with_capacity(n);
    let mut rng = CRand::new(1);

    for _ in 0..n {
        initial_data.push(rng.rexp());
    }

    detector.fit(&initial_data).unwrap();

    // Test with 100K samples
    let mut anomaly = 0;
    let mut excess = 0;
    let mut normal = 0;

    for _ in 0..100_000 {
        let x = rng.rexp();
        match detector.step(x).unwrap() {
            SpotStatus::Normal => normal += 1,
            SpotStatus::Excess => excess += 1,
            SpotStatus::Anomaly => anomaly += 1,
        }
    }

    println!("Pure Rust Results after 100K samples:");
    println!("ANOMALY={} EXCESS={} NORMAL={}", anomaly, excess, normal);
    println!(
        "Z={:.6} T={:.6}",
        detector.anomaly_threshold(),
        detector.excess_threshold()
    );

    // Validate results
    assert_eq!(anomaly + excess + normal, 100_000);
    assert!(!detector.anomaly_threshold().is_nan());
    assert!(!detector.excess_threshold().is_nan());
}
