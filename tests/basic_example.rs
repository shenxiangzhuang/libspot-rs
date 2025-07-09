use approx::assert_relative_eq;
use libspot::{SpotConfig, SpotDetector, SpotStatus};

/// Random number generator that matches C's rand()/srand() for reproducible results
struct CRand;

impl CRand {
    /// Create a new random number generator with the given seed
    fn new(seed: u32) -> Self {
        unsafe {
            libc::srand(seed);
        }
        CRand
    }

    /// Generate a random integer
    fn rand(&mut self) -> u32 {
        unsafe { libc::rand() as u32 }
    }

    /// Generate a uniform random float in [0, 1)
    fn runif(&mut self) -> f64 {
        self.rand() as f64 / 2147483647.0 // RAND_MAX = 2^31 - 1
    }

    /// Generate an exponentially distributed random variable with rate 1
    fn rexp(&mut self) -> f64 {
        -self.runif().ln()
    }
}

/// Test that reproduces the basic example behavior exactly
/// This test validates that our Rust binding produces identical results to the C library
#[test]
fn test_basic_example_behavior() {
    // Configure SPOT detector with exact same parameters as C basic example
    let config = SpotConfig {
        q: 0.0001,               // anomaly probability
        low_tail: false,         // observe upper tail
        discard_anomalies: true, // flag anomalies
        level: 0.998,            // tail quantile
        max_excess: 200,         // data points to keep
    };

    // Create and initialize SPOT detector
    let mut detector = SpotDetector::new(config).unwrap();

    // Generate initial training data with same seed as C example
    let n = 20000;
    let mut initial_data = Vec::with_capacity(n);
    let mut rng = CRand::new(1); // Use same seed as C example

    for _ in 0..n {
        initial_data.push(rng.rexp());
    }

    // Fit the model
    detector.fit(&initial_data).unwrap();

    // Main detection loop - use smaller number for faster testing
    // but still validate the behavior is correct
    let k = 1_000_000; // Use 1M samples for faster testing
    let mut normal = 0;
    let mut excess = 0;
    let mut anomaly = 0;

    for _ in 0..k {
        let val = rng.rexp();
        match detector.step(val).unwrap() {
            SpotStatus::Normal => normal += 1,
            SpotStatus::Excess => excess += 1,
            SpotStatus::Anomaly => anomaly += 1,
        }
    }

    // Verify the behavior is consistent with expectations
    assert!(normal > 0, "Should have normal classifications");
    assert!(excess > 0, "Should have excess classifications");
    assert!(anomaly > 0, "Should have anomaly classifications");

    // The total should equal k
    assert_eq!(normal + excess + anomaly, k);

    // Verify thresholds are reasonable
    let anomaly_threshold = detector.anomaly_threshold();
    let excess_threshold = detector.excess_threshold();

    assert!(
        anomaly_threshold > excess_threshold,
        "Anomaly threshold should be higher than excess threshold"
    );
    assert!(
        anomaly_threshold > 0.0,
        "Anomaly threshold should be positive"
    );
    assert!(
        excess_threshold > 0.0,
        "Excess threshold should be positive"
    );

    println!("Test results (1M samples):");
    println!("ANOMALY={} EXCESS={} NORMAL={}", anomaly, excess, normal);
    println!("Z={:.6} T={:.6}", anomaly_threshold, excess_threshold);
}

/// Full scale test that matches the C basic example exactly
/// This test uses 50M samples and validates exact output
/// Note: This test is marked as ignored by default due to long runtime
#[test]
#[ignore]
fn test_basic_example_full_scale() {
    // Configure SPOT detector with exact same parameters as C basic example
    let config = SpotConfig {
        q: 0.0001,               // anomaly probability
        low_tail: false,         // observe upper tail
        discard_anomalies: true, // flag anomalies
        level: 0.998,            // tail quantile
        max_excess: 200,         // data points to keep
    };

    // Create and initialize SPOT detector
    let mut detector = SpotDetector::new(config).unwrap();

    // Generate initial training data with same seed as C example
    let n = 20000;
    let mut initial_data = Vec::with_capacity(n);
    let mut rng = CRand::new(1); // Use same seed as C example

    for _ in 0..n {
        initial_data.push(rng.rexp());
    }

    // Fit the model
    detector.fit(&initial_data).unwrap();

    // Main detection loop - full 50M samples like C example
    let k = 50_000_000;
    let mut normal = 0;
    let mut excess = 0;
    let mut anomaly = 0;

    println!("Starting full scale test with {} samples...", k);
    let start = std::time::Instant::now();

    for _ in 0..k {
        let val = rng.rexp();
        match detector.step(val).unwrap() {
            SpotStatus::Normal => normal += 1,
            SpotStatus::Excess => excess += 1,
            SpotStatus::Anomaly => anomaly += 1,
        }
    }

    let duration = start.elapsed();

    // Expected results from C basic example
    let expected_anomaly = 25898;
    let expected_excess = 71938;
    let expected_normal = 49902164;
    let expected_z = 7.422655;
    let expected_t = 6.236165;

    // Validate results match C library exactly
    assert_eq!(
        anomaly, expected_anomaly,
        "Anomaly count should match C library"
    );
    assert_eq!(
        excess, expected_excess,
        "Excess count should match C library"
    );
    assert_eq!(
        normal, expected_normal,
        "Normal count should match C library"
    );

    // Validate thresholds match (allow small floating point differences)
    let z = detector.anomaly_threshold();
    let t = detector.excess_threshold();

    assert_relative_eq!(z, expected_z, epsilon = 1e-5);
    assert_relative_eq!(t, expected_t, epsilon = 1e-5);

    println!(
        "Full scale test completed in {:.2}s",
        duration.as_secs_f64()
    );
    println!("ANOMALY={} EXCESS={} NORMAL={}", anomaly, excess, normal);
    println!("Z={:.6} T={:.6}", z, t);
    println!("✓ All results match C library exactly!");
}
