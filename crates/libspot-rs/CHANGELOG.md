# Changelog

Changes to the `libspot-rs` crate are recorded here. Add pending changes under
`Unreleased` and move them into a version section when publishing a release.

## Unreleased

### Fixed

- Align P² input validation with C 3.1.0: reject training batches with fewer
  than five samples or any NaN, returning `SpotError::ExcessThresholdIsNaN`
  from `fit`. Also reject quantile probabilities outside `(0, 1)`, including
  NaN. Empirical initial-threshold selection is unaffected.

## 0.4.0-rc.3

### Fixed

- Correct P² marker updates for observations that become a new minimum or
  maximum, matching [upstream libspot #42](https://github.com/asiffer/libspot/pull/42)
  (released in libspot 3.1.0). This corrects initial SPOT thresholds, especially
  for monotonic training data. Models fitted with the default P² threshold can
  produce different thresholds and classifications than earlier releases.
  Existing serialized models retain their stored thresholds until refitted;
  empirical initial thresholds are unaffected.
