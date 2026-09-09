// Run identical public-API regression cases against bundled C 3.1.0 (via FFI)
// and pure Rust in this unpublished workspace member, without adding C
// dependencies to the pure Rust crate.
macro_rules! p2_input_tests {
    ($module:ident, $implementation:ident) => {
        mod $module {
            use $implementation::{SpotConfig, SpotDetector, SpotError};

            fn detector(low_tail: bool) -> SpotDetector {
                SpotDetector::new(SpotConfig {
                    q: 0.01,
                    level: 0.5,
                    low_tail,
                    ..SpotConfig::default()
                })
                .unwrap()
            }

            fn assert_rejected(data: &[f64]) {
                for low_tail in [false, true] {
                    let mut detector = detector(low_tail);
                    assert_eq!(
                        detector.fit(data),
                        Err(SpotError::ExcessThresholdIsNaN),
                        "low_tail={low_tail}, data={data:?}"
                    );
                    assert!(detector.excess_threshold().is_nan());
                    assert!(detector.anomaly_threshold().is_nan());
                }
            }

            #[test]
            fn rejects_short_training_data() {
                let data = [1.0, 2.0, 3.0, 4.0];
                for size in 0..=data.len() {
                    assert_rejected(&data[..size]);
                }
            }

            #[test]
            fn rejects_nan_in_initial_markers() {
                for index in 0..5 {
                    let mut data = [1.0, 2.0, 3.0, 4.0, 5.0];
                    data[index] = f64::NAN;
                    assert_rejected(&data);
                }
            }

            #[test]
            fn rejects_nan_after_initial_markers() {
                for index in 5..10 {
                    let mut data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0];
                    data[index] = f64::NAN;
                    assert_rejected(&data);
                }
            }

            #[test]
            fn accepts_five_training_samples() {
                for low_tail in [false, true] {
                    let mut detector = detector(low_tail);
                    detector.fit(&[5.0, 3.0, 4.0, 1.0, 2.0]).unwrap();
                    assert_eq!(detector.excess_threshold(), 3.0);
                    assert!(detector.anomaly_threshold().is_finite());
                }
            }
        }
    };
}

p2_input_tests!(c, libspot);
p2_input_tests!(rust, libspot_rs);
