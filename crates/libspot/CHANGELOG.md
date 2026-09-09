# Changelog

Changes to the `libspot` crate are recorded here. Add pending changes under
`Unreleased` and move them into a version section when publishing a release.

## Unreleased

## 3.1.0

### Changed

- Bundle [libspot C 3.1.0](https://github.com/asiffer/libspot/releases/tag/v3.1.0),
  including the P² extrema marker update fix and quantile input validation fixes.
  The Rust API is unchanged. Refitting models can produce different thresholds
  and classifications than 3.0.0, especially with monotonic training data.
- Consistently reject training data containing NaN with
  `SpotError::ExcessThresholdIsNaN`.
