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

const POSITIVE_ANOMALY_INDEX: usize = 80;
const NEGATIVE_ANOMALY_INDEX: usize = 160;

fn time_series_with_injected_anomalies() -> Vec<f64> {
    (0..240)
        .map(|i| match i {
            POSITIVE_ANOMALY_INDEX => 1_000_000.0,
            NEGATIVE_ANOMALY_INDEX => -1_000_000.0,
            _ => {
                let x = (i + 1000) as f64 * 0.031;
                x.sin() + 0.2 * (x * 0.37).cos()
            }
        })
        .collect()
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

#[test]
fn injected_time_series_anomalies_have_expected_scores_in_all_modes() {
    let series = time_series_with_injected_anomalies();
    let mut upper = detector(false);
    let mut lower = detector(true);
    let mut both_upper = detector(false);
    let mut both_lower = detector(true);

    let initial_upper_n = upper.n();
    let initial_lower_n = lower.n();
    let initial_both_upper_n = both_upper.n();
    let initial_both_lower_n = both_lower.n();
    let mut largest_normal_upper_score: f64 = 0.0;
    let mut largest_normal_lower_score: f64 = 0.0;
    let mut largest_normal_both_score: f64 = 0.0;

    for (index, value) in series.into_iter().enumerate() {
        // Scores are intentionally obtained before `step`: an online score must
        // describe the sample under the model fitted to preceding observations.
        let upper_score = upper.anomaly_score(value);
        let lower_score = lower.anomaly_score(value);
        let both_upper_score = both_upper.anomaly_score(value);
        let both_lower_score = both_lower.anomaly_score(value);
        let both_score = both_upper_score.max(both_lower_score);

        for score in [
            upper_score,
            lower_score,
            both_upper_score,
            both_lower_score,
            both_score,
        ] {
            assert!((0.0..=1.0).contains(&score));
        }
        assert_eq!(both_score, both_upper_score.max(both_lower_score));

        let upper_status = upper.step(value).unwrap();
        let lower_status = lower.step(value).unwrap();
        let both_upper_status = both_upper.step(value).unwrap();
        let both_lower_status = both_lower.step(value).unwrap();

        match index {
            POSITIVE_ANOMALY_INDEX => {
                assert_eq!(upper_score, 1.0);
                assert_eq!(lower_score, 0.0);
                assert_eq!(both_upper_score, 1.0);
                assert_eq!(both_lower_score, 0.0);
                assert_eq!(both_score, 1.0);
                assert_eq!(upper_status, SpotStatus::Anomaly);
                assert_eq!(lower_status, SpotStatus::Normal);
                assert_eq!(both_upper_status, SpotStatus::Anomaly);
                assert_eq!(both_lower_status, SpotStatus::Normal);
            }
            NEGATIVE_ANOMALY_INDEX => {
                assert_eq!(upper_score, 0.0);
                assert_eq!(lower_score, 1.0);
                assert_eq!(both_upper_score, 0.0);
                assert_eq!(both_lower_score, 1.0);
                assert_eq!(both_score, 1.0);
                assert_eq!(upper_status, SpotStatus::Normal);
                assert_eq!(lower_status, SpotStatus::Anomaly);
                assert_eq!(both_upper_status, SpotStatus::Normal);
                assert_eq!(both_lower_status, SpotStatus::Anomaly);
            }
            _ => {
                // A baseline point may legitimately be an EVT excess while
                // remaining below the anomaly threshold.
                assert_ne!(upper_status, SpotStatus::Anomaly);
                assert_ne!(lower_status, SpotStatus::Anomaly);
                assert_ne!(both_upper_status, SpotStatus::Anomaly);
                assert_ne!(both_lower_status, SpotStatus::Anomaly);
                largest_normal_upper_score = largest_normal_upper_score.max(upper_score);
                largest_normal_lower_score = largest_normal_lower_score.max(lower_score);
                largest_normal_both_score = largest_normal_both_score.max(both_score);
            }
        }
    }

    assert!(largest_normal_upper_score < 1.0);
    assert!(largest_normal_lower_score < 1.0);
    assert!(largest_normal_both_score < 1.0);

    // Each one-sided detector discards only the anomaly in its own tail. The
    // opposite-tail spike remains a normal observation and advances its state.
    let processed_normal_for_each_tail = 239;
    assert_eq!(upper.n(), initial_upper_n + processed_normal_for_each_tail);
    assert_eq!(lower.n(), initial_lower_n + processed_normal_for_each_tail);
    assert_eq!(
        both_upper.n(),
        initial_both_upper_n + processed_normal_for_each_tail
    );
    assert_eq!(
        both_lower.n(),
        initial_both_lower_n + processed_normal_for_each_tail
    );
}
