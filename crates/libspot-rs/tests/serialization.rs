//! Tests for serialization and deserialization of SPOT models
//!
//! These tests verify that the SPOT detector and its components can be
//! properly serialized and deserialized, which is essential for model
//! persistence and deployment scenarios.

#![cfg(feature = "serde")]

use approx::assert_relative_eq;
use libspot_rs::{Peaks, SpotConfig, SpotDetector, SpotError, SpotStatus, Tail, Ubend};

// ============================================================================
// SpotConfig Serialization Tests
// ============================================================================

#[test]
fn test_spot_config_json_roundtrip() {
    let original = SpotConfig {
        q: 0.001,
        low_tail: true,
        discard_anomalies: false,
        level: 0.99,
        max_excess: 150,
    };

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: SpotConfig = serde_json::from_str(&json).unwrap();

    assert_relative_eq!(deserialized.q, original.q);
    assert_eq!(deserialized.low_tail, original.low_tail);
    assert_eq!(deserialized.discard_anomalies, original.discard_anomalies);
    assert_relative_eq!(deserialized.level, original.level);
    assert_eq!(deserialized.max_excess, original.max_excess);
}

#[test]
fn test_spot_config_default_roundtrip() {
    let original = SpotConfig::default();

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: SpotConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized, original);
}

#[test]
fn test_spot_config_pretty_json() {
    let config = SpotConfig::default();
    let pretty_json = serde_json::to_string_pretty(&config).unwrap();

    // Verify it contains expected fields
    assert!(pretty_json.contains("\"q\""));
    assert!(pretty_json.contains("\"low_tail\""));
    assert!(pretty_json.contains("\"discard_anomalies\""));
    assert!(pretty_json.contains("\"level\""));
    assert!(pretty_json.contains("\"max_excess\""));
}

// ============================================================================
// SpotStatus Serialization Tests
// ============================================================================

#[test]
fn test_spot_status_roundtrip() {
    for status in [SpotStatus::Normal, SpotStatus::Excess, SpotStatus::Anomaly] {
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: SpotStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, status);
    }
}

// ============================================================================
// SpotError Serialization Tests
// ============================================================================

#[test]
fn test_spot_error_roundtrip() {
    let errors = [
        SpotError::MemoryAllocationFailed,
        SpotError::LevelOutOfBounds,
        SpotError::QOutOfBounds,
        SpotError::ExcessThresholdIsNaN,
        SpotError::AnomalyThresholdIsNaN,
        SpotError::DataIsNaN,
    ];

    for error in errors {
        let json = serde_json::to_string(&error).unwrap();
        let deserialized: SpotError = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, error);
    }
}

// ============================================================================
// Ubend Serialization Tests
// ============================================================================

#[test]
fn test_ubend_empty_roundtrip() {
    let original = Ubend::new(5).unwrap();

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: Ubend = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.size(), original.size());
    assert_eq!(deserialized.capacity(), original.capacity());
    assert_eq!(deserialized.is_filled(), original.is_filled());
}

#[test]
fn test_ubend_partial_filled_roundtrip() {
    let mut original = Ubend::new(5).unwrap();
    original.push(1.0);
    original.push(2.0);
    original.push(3.0);

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: Ubend = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.size(), original.size());
    assert_eq!(deserialized.capacity(), original.capacity());
    assert!(!deserialized.is_filled());

    // Verify data matches
    for i in 0..original.size() {
        assert_relative_eq!(deserialized.get(i).unwrap(), original.get(i).unwrap());
    }
}

#[test]
fn test_ubend_full_with_wraparound_roundtrip() {
    let mut original = Ubend::new(3).unwrap();
    // Fill and wrap around
    original.push(1.0);
    original.push(2.0);
    original.push(3.0);
    original.push(4.0); // Overwrites 1.0
    original.push(5.0); // Overwrites 2.0

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: Ubend = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.size(), 3);
    assert!(deserialized.is_filled());

    // Should contain [3.0, 4.0, 5.0] in insertion order
    let data: Vec<f64> = deserialized.iter().collect();
    assert_eq!(data, vec![3.0, 4.0, 5.0]);
}

// ============================================================================
// Peaks Serialization Tests
// ============================================================================

#[test]
fn test_peaks_empty_roundtrip() {
    let original = Peaks::new(10).unwrap();

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: Peaks = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.size(), 0);
    assert!(deserialized.mean().is_nan());
    assert!(deserialized.min().is_nan());
    assert!(deserialized.max().is_nan());
}

#[test]
fn test_peaks_with_data_roundtrip() {
    let mut original = Peaks::new(10).unwrap();
    for v in [1.0, 2.0, 3.0, 4.0, 5.0] {
        original.push(v);
    }

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: Peaks = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.size(), original.size());
    assert_relative_eq!(deserialized.mean(), original.mean());
    assert_relative_eq!(deserialized.variance(), original.variance());
    assert_relative_eq!(deserialized.min(), original.min());
    assert_relative_eq!(deserialized.max(), original.max());
}

// ============================================================================
// Tail Serialization Tests
// ============================================================================

#[test]
fn test_tail_empty_roundtrip() {
    let original = Tail::new(10).unwrap();

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: Tail = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.size(), 0);
    assert!(deserialized.gamma().is_nan());
    assert!(deserialized.sigma().is_nan());
}

#[test]
fn test_tail_fitted_roundtrip() {
    let mut original = Tail::new(10).unwrap();
    for v in [0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0] {
        original.push(v);
    }
    original.fit();

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: Tail = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.size(), original.size());
    assert_relative_eq!(deserialized.gamma(), original.gamma());
    assert_relative_eq!(deserialized.sigma(), original.sigma());
}

// ============================================================================
// SpotDetector Serialization Tests
// ============================================================================

#[test]
fn test_spot_detector_new_roundtrip() {
    let config = SpotConfig::default();
    let original = SpotDetector::new(config).unwrap();

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: SpotDetector = serde_json::from_str(&json).unwrap();

    // Check basic properties
    assert_eq!(deserialized.n(), original.n());
    assert_eq!(deserialized.nt(), original.nt());
    assert!(deserialized.anomaly_threshold().is_nan());
    assert!(deserialized.excess_threshold().is_nan());
}

#[test]
fn test_spot_detector_fitted_roundtrip() {
    let config = SpotConfig::default();
    let mut original = SpotDetector::new(config).unwrap();

    // Fit with training data
    let training_data: Vec<f64> = (0..1000).map(|i| (i as f64) / 100.0).collect();
    original.fit(&training_data).unwrap();

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: SpotDetector = serde_json::from_str(&json).unwrap();

    // Check fitted properties are preserved
    assert_eq!(deserialized.n(), original.n());
    assert_eq!(deserialized.nt(), original.nt());
    assert_relative_eq!(
        deserialized.anomaly_threshold(),
        original.anomaly_threshold()
    );
    assert_relative_eq!(deserialized.excess_threshold(), original.excess_threshold());

    // Check tail parameters
    let (orig_gamma, orig_sigma) = original.tail_parameters();
    let (deser_gamma, deser_sigma) = deserialized.tail_parameters();
    assert_relative_eq!(deser_gamma, orig_gamma);
    assert_relative_eq!(deser_sigma, orig_sigma);
}

#[test]
fn test_spot_detector_functional_after_deserialization() {
    let config = SpotConfig::default();
    let mut original = SpotDetector::new(config).unwrap();

    // Fit and process some data
    let training_data: Vec<f64> = (0..1000).map(|i| (i as f64) / 100.0).collect();
    original.fit(&training_data).unwrap();

    // Serialize
    let json = serde_json::to_string(&original).unwrap();

    // Deserialize
    let _deserialized: SpotDetector = serde_json::from_str(&json).unwrap();

    // Verify the deserialized detector produces same results as original
    let test_values = [5.0, 10.0, 50.0, 100.0, 500.0];

    // We need fresh detectors since step mutates state
    let mut orig_fresh = SpotDetector::new(SpotConfig::default()).unwrap();
    let mut deser_fresh: SpotDetector = serde_json::from_str(&json).unwrap();

    orig_fresh.fit(&training_data).unwrap();

    // Test that both produce same thresholds after same operations
    assert_relative_eq!(
        orig_fresh.anomaly_threshold(),
        deser_fresh.anomaly_threshold()
    );
    assert_relative_eq!(
        orig_fresh.excess_threshold(),
        deser_fresh.excess_threshold()
    );

    // Test step produces same status
    for &val in &test_values {
        let orig_status = orig_fresh.step(val);
        let deser_status = deser_fresh.step(val);

        // Both should succeed or fail together
        assert_eq!(orig_status.is_ok(), deser_status.is_ok());

        if let (Ok(os), Ok(ds)) = (orig_status, deser_status) {
            assert_eq!(os, ds, "Status mismatch for value {}", val);
        }
    }
}

#[test]
fn test_spot_detector_low_tail_roundtrip() {
    let config = SpotConfig {
        low_tail: true,
        ..SpotConfig::default()
    };
    let mut original = SpotDetector::new(config).unwrap();

    let training_data: Vec<f64> = (0..1000).map(|i| (i as f64) / 100.0).collect();
    original.fit(&training_data).unwrap();

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: SpotDetector = serde_json::from_str(&json).unwrap();

    // Verify config was preserved
    let deser_config = deserialized.config().unwrap();
    assert!(deser_config.low_tail);
}

#[test]
fn test_spot_detector_pretty_json_output() {
    let config = SpotConfig::default();
    let mut spot = SpotDetector::new(config).unwrap();
    let training_data: Vec<f64> = (0..100).map(|i| i as f64).collect();
    spot.fit(&training_data).unwrap();

    let pretty_json = serde_json::to_string_pretty(&spot).unwrap();

    // Verify it's readable and contains expected structure
    assert!(pretty_json.contains("\"q\""));
    assert!(pretty_json.contains("\"level\""));
    assert!(pretty_json.contains("\"tail\""));
    assert!(pretty_json.contains("\"anomaly_threshold\""));
    assert!(pretty_json.contains("\"excess_threshold\""));
}

// ============================================================================
// Model Persistence Workflow Tests
// ============================================================================

#[test]
fn test_model_save_load_workflow() {
    // Simulate a real-world workflow: train, save, load, continue processing

    // Step 1: Train a model
    let config = SpotConfig {
        q: 0.001,
        level: 0.98,
        max_excess: 100,
        ..SpotConfig::default()
    };
    let mut model = SpotDetector::new(config).unwrap();

    let training_data: Vec<f64> = (0..500).map(|i| (i as f64) * 0.1).collect();
    model.fit(&training_data).unwrap();

    // Process some streaming data
    for i in 500..600 {
        let _ = model.step((i as f64) * 0.1);
    }

    // Step 2: Save the model (serialize)
    let saved_model = serde_json::to_string(&model).unwrap();

    // Step 3: Later, load the model (deserialize)
    let mut loaded_model: SpotDetector = serde_json::from_str(&saved_model).unwrap();

    // Step 4: Continue processing with the loaded model
    // Verify state was preserved
    assert_eq!(loaded_model.n(), model.n());
    assert_eq!(loaded_model.nt(), model.nt());

    // Continue processing
    let status = loaded_model.step(100.0);
    assert!(status.is_ok());
}

#[test]
fn test_multiple_serialization_cycles() {
    let config = SpotConfig::default();
    let mut model = SpotDetector::new(config).unwrap();

    let data: Vec<f64> = (0..200).map(|i| i as f64).collect();
    model.fit(&data).unwrap();

    // Serialize and deserialize multiple times
    let mut current = model;
    for _ in 0..5 {
        let json = serde_json::to_string(&current).unwrap();
        current = serde_json::from_str(&json).unwrap();

        // Process some data
        let _ = current.step(50.0);
    }

    // Should still be functional
    let status = current.step(50.0);
    assert!(status.is_ok());
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_deserialize_invalid_json() {
    let invalid_json = "{\"invalid\": \"data\"}";
    let result: Result<SpotDetector, _> = serde_json::from_str(invalid_json);
    assert!(result.is_err());
}

#[test]
fn test_nan_values_serialization() {
    // Test that NaN values are handled correctly
    let config = SpotConfig::default();
    let original = SpotDetector::new(config).unwrap();

    // Before fitting, thresholds are NaN
    assert!(original.anomaly_threshold().is_nan());
    assert!(original.excess_threshold().is_nan());

    let json = serde_json::to_string(&original).unwrap();
    let deserialized: SpotDetector = serde_json::from_str(&json).unwrap();

    // NaN values should be preserved
    assert!(deserialized.anomaly_threshold().is_nan());
    assert!(deserialized.excess_threshold().is_nan());
}

// ============================================================================
// Property-based Tests for Serialization Equivalence
// ============================================================================

mod proptest_serde {
    use super::*;
    use proptest::prelude::*;

    /// Strategy for generating valid SpotConfig
    fn spot_config_strategy() -> impl Strategy<Value = SpotConfig> {
        // level must be in (0, 1), typically close to 1
        // q must be in (0, 1 - level)
        (
            0.9..0.999f64,       // level
            50usize..500,        // max_excess
            proptest::bool::ANY, // low_tail
            proptest::bool::ANY, // discard_anomalies
        )
            .prop_flat_map(|(level, max_excess, low_tail, discard_anomalies)| {
                // q must be < (1 - level) and > 0
                let max_q = (1.0 - level) * 0.9; // Leave some margin
                let min_q = 0.00001;
                (
                    Just(level),
                    min_q..max_q,
                    Just(max_excess),
                    Just(low_tail),
                    Just(discard_anomalies),
                )
            })
            .prop_map(
                |(level, q, max_excess, low_tail, discard_anomalies)| SpotConfig {
                    q,
                    level,
                    max_excess,
                    low_tail,
                    discard_anomalies,
                },
            )
    }

    /// Strategy for generating training data
    fn training_data_strategy() -> impl Strategy<Value = Vec<f64>> {
        prop::collection::vec(0.0..100.0f64, 100..500)
    }

    /// Strategy for generating test values
    fn test_values_strategy() -> impl Strategy<Value = Vec<f64>> {
        prop::collection::vec(-10.0..200.0f64, 10..50)
    }

    proptest! {
        /// Property: SpotConfig serialization roundtrip preserves all fields
        #[test]
        fn prop_spot_config_roundtrip(config in spot_config_strategy()) {
            let json = serde_json::to_string(&config).unwrap();
            let loaded: SpotConfig = serde_json::from_str(&json).unwrap();

            prop_assert!((loaded.q - config.q).abs() < 1e-10);
            prop_assert!((loaded.level - config.level).abs() < 1e-10);
            prop_assert_eq!(loaded.max_excess, config.max_excess);
            prop_assert_eq!(loaded.low_tail, config.low_tail);
            prop_assert_eq!(loaded.discard_anomalies, config.discard_anomalies);
        }

        /// Property: Fitted SpotDetector state is preserved after serialization
        #[test]
        fn prop_fitted_detector_state_preserved(
            config in spot_config_strategy(),
            training_data in training_data_strategy()
        ) {
            let mut original = SpotDetector::new(config).unwrap();
            original.fit(&training_data).unwrap();

            let json = serde_json::to_string(&original).unwrap();
            let loaded: SpotDetector = serde_json::from_str(&json).unwrap();

            // Verify state preservation
            prop_assert_eq!(loaded.n(), original.n());
            prop_assert_eq!(loaded.nt(), original.nt());
            prop_assert!((loaded.anomaly_threshold() - original.anomaly_threshold()).abs() < 1e-10);
            prop_assert!((loaded.excess_threshold() - original.excess_threshold()).abs() < 1e-10);

            // Verify tail parameters
            let (orig_gamma, orig_sigma) = original.tail_parameters();
            let (loaded_gamma, loaded_sigma) = loaded.tail_parameters();
            prop_assert!((loaded_gamma - orig_gamma).abs() < 1e-10);
            prop_assert!((loaded_sigma - orig_sigma).abs() < 1e-10);

            // Verify config
            let orig_config = original.config().unwrap();
            let loaded_config = loaded.config().unwrap();
            prop_assert!((loaded_config.q - orig_config.q).abs() < 1e-10);
            prop_assert!((loaded_config.level - orig_config.level).abs() < 1e-10);
            prop_assert_eq!(loaded_config.max_excess, orig_config.max_excess);
            prop_assert_eq!(loaded_config.low_tail, orig_config.low_tail);
            prop_assert_eq!(loaded_config.discard_anomalies, orig_config.discard_anomalies);
        }

        /// Property: Loaded detector produces identical detection results
        #[test]
        fn prop_detection_behavior_identical(
            config in spot_config_strategy(),
            training_data in training_data_strategy(),
            test_values in test_values_strategy()
        ) {
            // Create and fit original detector
            let mut original = SpotDetector::new(config.clone()).unwrap();
            original.fit(&training_data).unwrap();

            // Serialize and deserialize
            let json = serde_json::to_string(&original).unwrap();
            let mut loaded: SpotDetector = serde_json::from_str(&json).unwrap();

            // Both detectors should produce identical results for the same inputs
            for value in test_values {
                let orig_result = original.step(value);
                let loaded_result = loaded.step(value);

                // Both should succeed or fail together
                prop_assert_eq!(orig_result.is_ok(), loaded_result.is_ok());

                if let (Ok(orig_status), Ok(loaded_status)) = (orig_result, loaded_result) {
                    prop_assert_eq!(
                        orig_status, loaded_status,
                        "Detection mismatch for value {}: original={:?}, loaded={:?}",
                        value, orig_status, loaded_status
                    );
                }

                // After each step, states should still match
                prop_assert_eq!(loaded.n(), original.n());
                prop_assert_eq!(loaded.nt(), original.nt());

                // Compare thresholds, handling NaN cases
                let orig_thresh = original.anomaly_threshold();
                let loaded_thresh = loaded.anomaly_threshold();
                if orig_thresh.is_nan() {
                    prop_assert!(loaded_thresh.is_nan());
                } else {
                    // Use relative tolerance for large values
                    let diff = (loaded_thresh - orig_thresh).abs();
                    let rel_diff = diff / orig_thresh.abs().max(1.0);
                    prop_assert!(
                        rel_diff < 1e-9,
                        "Threshold mismatch: original={}, loaded={}, rel_diff={}",
                        orig_thresh, loaded_thresh, rel_diff
                    );
                }
            }
        }

        /// Property: Multiple serialization cycles preserve equivalence
        #[test]
        fn prop_multiple_serialization_cycles(
            config in spot_config_strategy(),
            training_data in training_data_strategy(),
            num_cycles in 1usize..5
        ) {
            let mut detector = SpotDetector::new(config).unwrap();
            detector.fit(&training_data).unwrap();

            let original_threshold = detector.anomaly_threshold();
            let original_n = detector.n();
            let original_nt = detector.nt();

            // Perform multiple serialize/deserialize cycles
            for _ in 0..num_cycles {
                let json = serde_json::to_string(&detector).unwrap();
                detector = serde_json::from_str(&json).unwrap();
            }

            // State should be preserved after all cycles
            prop_assert!((detector.anomaly_threshold() - original_threshold).abs() < 1e-10);
            prop_assert_eq!(detector.n(), original_n);
            prop_assert_eq!(detector.nt(), original_nt);
        }

        /// Property: Ubend serialization preserves data and state
        #[test]
        fn prop_ubend_roundtrip(
            capacity in 5usize..100,
            values in prop::collection::vec(0.0..1000.0f64, 1..200)
        ) {
            let mut original = Ubend::new(capacity).unwrap();
            for v in &values {
                original.push(*v);
            }

            let json = serde_json::to_string(&original).unwrap();
            let loaded: Ubend = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(loaded.size(), original.size());
            prop_assert_eq!(loaded.capacity(), original.capacity());
            prop_assert_eq!(loaded.is_filled(), original.is_filled());

            // Verify data matches
            for i in 0..original.size() {
                let orig_val = original.get(i).unwrap();
                let loaded_val = loaded.get(i).unwrap();
                prop_assert!((orig_val - loaded_val).abs() < 1e-10);
            }
        }

        /// Property: Peaks serialization preserves statistics
        #[test]
        fn prop_peaks_roundtrip(
            capacity in 5usize..100,
            values in prop::collection::vec(0.1..1000.0f64, 1..200)  // Avoid 0 for variance
        ) {
            let mut original = Peaks::new(capacity).unwrap();
            for v in &values {
                original.push(*v);
            }

            let json = serde_json::to_string(&original).unwrap();
            let loaded: Peaks = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(loaded.size(), original.size());
            prop_assert!((loaded.mean() - original.mean()).abs() < 1e-10);
            prop_assert!((loaded.variance() - original.variance()).abs() < 1e-6);
            prop_assert!((loaded.min() - original.min()).abs() < 1e-10);
            prop_assert!((loaded.max() - original.max()).abs() < 1e-10);
        }

        /// Property: Tail serialization preserves GPD parameters after fitting
        #[test]
        fn prop_tail_roundtrip(
            capacity in 10usize..100,
            values in prop::collection::vec(0.1..100.0f64, 10..200)
        ) {
            let mut original = Tail::new(capacity).unwrap();
            for v in &values {
                original.push(*v);
            }
            original.fit();

            let json = serde_json::to_string(&original).unwrap();
            let loaded: Tail = serde_json::from_str(&json).unwrap();

            prop_assert_eq!(loaded.size(), original.size());
            prop_assert!((loaded.gamma() - original.gamma()).abs() < 1e-10);
            prop_assert!((loaded.sigma() - original.sigma()).abs() < 1e-10);
        }
    }
}
