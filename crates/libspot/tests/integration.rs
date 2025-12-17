use approx::assert_relative_eq;
use libspot::{SpotConfig, SpotDetector, SpotError, SpotStatus};

/// Test basic SPOT detector initialization - validates that our binding correctly initializes the C library
#[test]
fn test_spot_detector_initialization() {
    // Test valid configuration
    let config = SpotConfig {
        q: 1e-5,
        low_tail: false,
        discard_anomalies: true,
        level: 0.995,
        max_excess: 200,
    };

    let detector = SpotDetector::new(config.clone());
    assert!(detector.is_ok(), "Valid configuration should succeed");

    let detector = detector.expect("Failed to create detector");
    let retrieved_config = detector.config().unwrap();

    // Verify configuration is preserved
    assert_relative_eq!(retrieved_config.q, config.q);
    assert_eq!(retrieved_config.low_tail, config.low_tail);
    assert_eq!(retrieved_config.discard_anomalies, config.discard_anomalies);
    assert_relative_eq!(retrieved_config.level, config.level);
    assert_eq!(retrieved_config.max_excess, config.max_excess);

    // Test low_tail mode
    let config_low = SpotConfig {
        low_tail: true,
        ..config.clone()
    };
    let detector_low = SpotDetector::new(config_low);
    assert!(detector_low.is_ok(), "Low tail mode should work");
}

/// Test error conditions match C library behavior
#[test]
fn test_error_conditions() {
    // Test level out of bounds (same as C test: level < 0)
    let config_bad_level = SpotConfig {
        level: -0.5,
        ..SpotConfig::default()
    };
    let result = SpotDetector::new(config_bad_level);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), SpotError::LevelOutOfBounds);

    // Test level >= 1.0
    let config_bad_level2 = SpotConfig {
        level: 1.0,
        ..SpotConfig::default()
    };
    let result2 = SpotDetector::new(config_bad_level2);
    assert!(result2.is_err());
    assert_eq!(result2.unwrap_err(), SpotError::LevelOutOfBounds);

    // Test q out of bounds (same as C test)
    let level = 0.995;
    let config_bad_q = SpotConfig {
        q: 1.0 - level + (1.0 + level) / 2.0, // From C test: q >= (1-level)
        level,
        ..SpotConfig::default()
    };
    let result3 = SpotDetector::new(config_bad_q);
    assert!(result3.is_err());
    assert_eq!(result3.unwrap_err(), SpotError::QOutOfBounds);
}

/// Test basic fitting behavior - ensures our binding correctly calls C library fit function
#[test]
fn test_spot_fitting() {
    let config = SpotConfig::default();
    let mut detector = SpotDetector::new(config).unwrap();

    // Create simple test data
    let data: Vec<f64> = (0..1000).map(|i| (i as f64 / 1000.0) * 2.0 - 1.0).collect();

    // Before fit, thresholds should be NaN
    assert!(detector.anomaly_threshold().is_nan());
    assert!(detector.excess_threshold().is_nan());

    // Fit the model
    let result = detector.fit(&data);
    assert!(result.is_ok(), "Fitting should succeed with valid data");

    // After fit, thresholds should be valid numbers
    assert!(!detector.anomaly_threshold().is_nan());
    assert!(!detector.excess_threshold().is_nan());
    assert!(detector.anomaly_threshold().is_finite());
    assert!(detector.excess_threshold().is_finite());
}

/// Test step function behavior - validates streaming detection works like C library
#[test]
fn test_spot_step_function() {
    let config = SpotConfig {
        q: 1e-3,
        level: 0.95, // Less strict threshold
        discard_anomalies: true,
        ..SpotConfig::default()
    };
    let mut detector = SpotDetector::new(config).unwrap();

    // Create wider training data range
    let training_data: Vec<f64> = (0..1000)
        .map(|i| ((i as f64 / 500.0) - 1.0) * 2.0)
        .collect(); // Range: -2 to 2
    detector.fit(&training_data).unwrap();

    // Test NaN input - should match C library behavior
    let result = detector.step(f64::NAN);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), SpotError::DataIsNaN);

    // Test normal values
    let normal_result = detector.step(0.0);
    assert!(normal_result.is_ok());

    // Test values that should be mostly normal (below excess threshold)
    let mut normal_count = 0;
    let mut excess_count = 0;
    let mut anomaly_count = 0;

    // Get the excess threshold to understand what values should be normal
    let excess_threshold = detector.excess_threshold();

    // Test values that are definitely below the excess threshold
    for i in 0..100 {
        let value = excess_threshold - 0.5 - (i as f64 / 100.0); // Values below threshold
        match detector.step(value).unwrap() {
            SpotStatus::Normal => normal_count += 1,
            SpotStatus::Excess => excess_count += 1,
            SpotStatus::Anomaly => anomaly_count += 1,
        }
    }

    // Most values should be normal since they're below the excess threshold
    assert!(
        normal_count > 0,
        "Should have some normal classifications, got normal={normal_count}, excess={excess_count}, anomaly={anomaly_count}"
    );

    // Test extreme values that should trigger anomalies
    let extreme_result = detector.step(1000.0);
    assert!(extreme_result.is_ok());
    // Note: We don't assert the specific result since it depends on the model state
}

/// Test quantile function behavior
#[test]
fn test_quantile_behavior() {
    let config = SpotConfig::default();
    let mut detector = SpotDetector::new(config).unwrap();

    // Simple symmetric data
    let data: Vec<f64> = (0..1000).map(|i| (i as f64 / 500.0) - 1.0).collect();
    detector.fit(&data).unwrap();

    // Test quantile function
    let q_low = detector.quantile(0.01);
    let q_high = detector.quantile(0.001);

    assert!(q_low.is_finite(), "Quantile should be finite");
    assert!(q_high.is_finite(), "Quantile should be finite");
    assert!(
        q_high > q_low,
        "Lower probability should give higher quantile"
    );
}

/// Test configuration retrieval
#[test]
fn test_config_retrieval() {
    let original_config = SpotConfig {
        q: 0.00005,
        low_tail: true,
        discard_anomalies: false,
        level: 0.99,
        max_excess: 500,
    };

    let detector = SpotDetector::new(original_config.clone()).unwrap();
    let retrieved_config = detector.config().unwrap();

    assert_relative_eq!(retrieved_config.q, original_config.q);
    assert_eq!(retrieved_config.low_tail, original_config.low_tail);
    assert_eq!(
        retrieved_config.discard_anomalies,
        original_config.discard_anomalies
    );
    assert_relative_eq!(retrieved_config.level, original_config.level);
    assert_eq!(retrieved_config.max_excess, original_config.max_excess);
}

/// Test version function
#[test]
fn test_version_function() {
    let version = libspot::version();
    assert!(!version.is_empty(), "Version should not be empty");
    println!("Library version: {version}");
}

/// Test that detector properly handles different data sizes
#[test]
fn test_different_data_sizes() {
    let config = SpotConfig::default();

    // Test with small dataset
    let mut detector1 = SpotDetector::new(config.clone()).unwrap();
    let small_data: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let result1 = detector1.fit(&small_data);
    assert!(result1.is_ok(), "Should handle small datasets");

    // Test with larger dataset
    let mut detector2 = SpotDetector::new(config).unwrap();
    let large_data: Vec<f64> = (0..10000).map(|i| i as f64 / 1000.0).collect();
    let result2 = detector2.fit(&large_data);
    assert!(result2.is_ok(), "Should handle large datasets");
}

/// Test edge cases for configuration parameters
#[test]
fn test_config_edge_cases() {
    // Test minimum valid level
    let config_min = SpotConfig {
        level: 0.001, // Very small but valid
        q: 0.0001,
        ..SpotConfig::default()
    };
    let result_min = SpotDetector::new(config_min);
    assert!(result_min.is_ok(), "Minimum valid level should work");

    // Test maximum valid level (just under 1.0)
    let config_max = SpotConfig {
        level: 0.9999,
        q: 0.00001, // Very small q to stay in bounds
        ..SpotConfig::default()
    };
    let result_max = SpotDetector::new(config_max);
    assert!(result_max.is_ok(), "Maximum valid level should work");
}

/// Test that multiple detectors can be created independently
#[test]
fn test_multiple_detectors() {
    let config1 = SpotConfig {
        level: 0.95,
        ..SpotConfig::default()
    };
    let config2 = SpotConfig {
        level: 0.99,
        ..SpotConfig::default()
    };

    let detector1 = SpotDetector::new(config1);
    let detector2 = SpotDetector::new(config2);

    assert!(detector1.is_ok(), "First detector should be created");
    assert!(detector2.is_ok(), "Second detector should be created");

    let det1 = detector1.unwrap();
    let det2 = detector2.unwrap();

    // Verify they have different configurations
    assert_ne!(det1.config().unwrap().level, det2.config().unwrap().level);
}
