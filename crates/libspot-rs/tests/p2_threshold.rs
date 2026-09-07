use libspot_rs::{SpotConfig, SpotDetector};

#[test]
fn monotonic_training_updates_p2_threshold_for_both_tails() {
    let ascending: Vec<f64> = (1..=20).map(f64::from).collect();
    let descending: Vec<f64> = ascending.iter().rev().copied().collect();

    for low_tail in [false, true] {
        for (data, expected) in [(&ascending, 10.0), (&descending, 11.0)] {
            let mut detector = SpotDetector::new(SpotConfig {
                q: 0.01,
                level: 0.5,
                low_tail,
                ..SpotConfig::default()
            })
            .unwrap();
            detector.fit(data).unwrap();

            assert_eq!(detector.excess_threshold(), expected);
            assert!(detector.anomaly_threshold().is_finite());
        }
    }
}
