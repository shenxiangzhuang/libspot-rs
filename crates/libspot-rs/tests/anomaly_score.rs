use approx::assert_relative_eq;
use libspot_rs::{SpotConfig, SpotDetector, SpotEstimator, SpotInitialThreshold, SpotStatus};

fn detector(low_tail: bool) -> SpotDetector {
    let config = SpotConfig {
        q: 0.001,
        low_tail,
        discard_anomalies: true,
        level: 0.9,
        max_excess: 200,
    };
    let mut detector = SpotDetector::new_with_options(
        config,
        SpotEstimator::Best,
        SpotInitialThreshold::Empirical,
    )
    .unwrap();
    let training_data: Vec<f64> = (0..1000)
        .map(|i| {
            let x = i as f64 * 0.031;
            x.sin() + 0.2 * (x * 0.37).cos()
        })
        .collect();
    detector.fit(&training_data).unwrap();
    detector
}

#[test]
fn public_anomaly_score_is_a_bounded_tail_percentile() {
    for low_tail in [false, true] {
        let detector = detector(low_tail);
        let tail_fraction = detector.nt() as f64 / detector.n() as f64;

        assert_eq!(detector.anomaly_score(detector.excess_threshold()), 0.0);
        assert_relative_eq!(
            detector.anomaly_score(detector.anomaly_threshold()),
            1.0 - detector.config().unwrap().q / tail_fraction,
            epsilon = 1e-8
        );
    }
}

#[test]
fn two_sided_score_uses_max_without_changing_independent_state_management() {
    let mut upper = detector(false);
    let mut lower = detector(true);
    let value = 1_000_000.0;

    let upper_n = upper.n();
    let lower_n = lower.n();
    let upper_score = upper.anomaly_score(value);
    let lower_score = lower.anomaly_score(value);
    let both_score = upper_score.max(lower_score);

    assert_eq!(both_score, upper_score);
    assert_eq!(both_score, 1.0);
    assert_eq!(lower_score, 0.0);
    assert_eq!(upper.n(), upper_n, "scoring must be read-only");
    assert_eq!(lower.n(), lower_n, "scoring must be read-only");

    assert_eq!(upper.step(value).unwrap(), SpotStatus::Anomaly);
    assert_eq!(lower.step(value).unwrap(), SpotStatus::Normal);

    // Preserve the existing composition behavior: each one-sided detector
    // owns and updates its state independently. With discarded upper-tail
    // anomalies, only the lower detector advances its observation count.
    assert_eq!(upper.n(), upper_n);
    assert_eq!(lower.n(), lower_n + 1);
}
