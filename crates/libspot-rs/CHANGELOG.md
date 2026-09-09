# Changelog

## Unreleased

- chore: organize crates and compatibility tests into a Cargo workspace by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#36](https://github.com/shenxiangzhuang/libspot-rs/pull/36)
- fix(libspot-rs): align P2 input validation with C 3.1.0 by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#36](https://github.com/shenxiangzhuang/libspot-rs/pull/36)
- ci: restore direct C/FFI/pure Rust consistency checks by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#35](https://github.com/shenxiangzhuang/libspot-rs/pull/35)

## [0.4.0-rc.3](https://github.com/shenxiangzhuang/libspot-rs/releases/tag/libspot-rs-v0.4.0-rc.3) - 2026-09-07

- fix(libspot-rs): correct P2 extrema updates and release 0.4.0-rc.3 by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#34](https://github.com/shenxiangzhuang/libspot-rs/pull/34)

## [0.4.0-rc.2](https://github.com/shenxiangzhuang/libspot-rs/releases/tag/libspot-rs-v0.4.0-rc.2) - 2026-08-05

- feat(libspot-rs): add bounded EVT anomaly scores and rendered equations by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#32](https://github.com/shenxiangzhuang/libspot-rs/pull/32)
- chore(libspot-rs): release 0.4.0-rc.2 by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#33](https://github.com/shenxiangzhuang/libspot-rs/pull/33)

## [0.4.0-rc.1](https://github.com/shenxiangzhuang/libspot-rs/releases/tag/libspot-rs-v0.4.0-rc.1) - 2026-06-03

- feat(libspot-rs): add MOM, empirical-threshold and strict-excess options by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#30](https://github.com/shenxiangzhuang/libspot-rs/pull/30)
- feat(libspot-rs): add observe_normal() for skipped samples by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#30](https://github.com/shenxiangzhuang/libspot-rs/pull/30)
- fix(libspot-rs): clear the learned tail before repeated fits by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#30](https://github.com/shenxiangzhuang/libspot-rs/pull/30)
- chore(libspot-rs): release 0.4.0-rc.1 by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#31](https://github.com/shenxiangzhuang/libspot-rs/pull/31)

## [0.3.0](https://github.com/shenxiangzhuang/libspot-rs/releases/tag/libspot-rs-v0.3.0) - 2026-05-02

- feat(libspot-rs): add SpotDetector::reset() for API parity by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#28](https://github.com/shenxiangzhuang/libspot-rs/pull/28)
- chore(libspot-rs): release 0.3.0 by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#29](https://github.com/shenxiangzhuang/libspot-rs/pull/29)
- ci: post hyperfine benchmarks as PR comments by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#27](https://github.com/shenxiangzhuang/libspot-rs/pull/27)
- ci: add three-implementation hyperfine benchmarks by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#26](https://github.com/shenxiangzhuang/libspot-rs/pull/26)
- ci: verify C, Rust FFI and pure Rust produce identical example output by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#25](https://github.com/shenxiangzhuang/libspot-rs/pull/25)
- docs: add the Rust-themed logo by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#24](https://github.com/shenxiangzhuang/libspot-rs/pull/24)

## [0.2.1](https://github.com/shenxiangzhuang/libspot-rs/releases/tag/libspot-rs-v0.2.1) - 2026-02-02

- fix(libspot-rs): align tail estimator order with C by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#21](https://github.com/shenxiangzhuang/libspot-rs/pull/21)
- chore(libspot-rs): release 0.2.1 by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#22](https://github.com/shenxiangzhuang/libspot-rs/pull/22)

## [0.2.0](https://github.com/shenxiangzhuang/libspot-rs/releases/tag/libspot-rs-v0.2.0) - 2025-12-18

- feat(libspot-rs): support model serialization and deserialization by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#16](https://github.com/shenxiangzhuang/libspot-rs/pull/16)
- chore(libspot-rs): release 0.2.0 by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#20](https://github.com/shenxiangzhuang/libspot-rs/pull/20)
- docs: fix crate documentation links by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#19](https://github.com/shenxiangzhuang/libspot-rs/pull/19)
- docs: simplify README usage examples by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#14](https://github.com/shenxiangzhuang/libspot-rs/pull/14)
- docs: remove the obsolete CI badge by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#11](https://github.com/shenxiangzhuang/libspot-rs/pull/11)

## [0.1.0](https://github.com/shenxiangzhuang/libspot-rs/releases/tag/libspot-rs-v0.1.0) - 2025-08-18

- feat(libspot-rs): introduce the pure Rust implementation as a separate crate by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) and [**@Copilot**](https://github.com/apps/copilot-swe-agent) in [#10](https://github.com/shenxiangzhuang/libspot-rs/pull/10)
