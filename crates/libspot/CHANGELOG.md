# Changelog

## Unreleased

- chore: organize crates and compatibility tests into a Cargo workspace by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#36](https://github.com/shenxiangzhuang/libspot-rs/pull/36)

## [3.1.0](https://github.com/shenxiangzhuang/libspot-rs/releases/tag/libspot-v3.1.0) - 2026-09-07

- chore(libspot): update to upstream 3.1.0 and restore shared CI by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#35](https://github.com/shenxiangzhuang/libspot-rs/pull/35)
- ci: post hyperfine benchmarks as PR comments by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#27](https://github.com/shenxiangzhuang/libspot-rs/pull/27)

## [3.0.0](https://github.com/shenxiangzhuang/libspot-rs/releases/tag/libspot-v3.0.0) - 2026-05-02

- feat(libspot): migrate to the C 3.0.0 API with caller-owned buffers and reset by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#26](https://github.com/shenxiangzhuang/libspot-rs/pull/26)
- ci: add three-implementation hyperfine benchmarks by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#26](https://github.com/shenxiangzhuang/libspot-rs/pull/26)
- ci: verify C, Rust FFI and pure Rust produce identical example output by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#25](https://github.com/shenxiangzhuang/libspot-rs/pull/25)
- docs: add the Rust-themed logo by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#24](https://github.com/shenxiangzhuang/libspot-rs/pull/24)
- docs: update cross-implementation benchmarks by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#21](https://github.com/shenxiangzhuang/libspot-rs/pull/21)

## [2.0.0-beta.6.0](https://github.com/shenxiangzhuang/libspot-rs/releases/tag/libspot%28ffi%29-v2.0.0-beta.6.0) - 2025-12-18

The release tag points to #15; its revert and follow-up fixes (#17–#19) are included below.

- docs: fix crate documentation links by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#19](https://github.com/shenxiangzhuang/libspot-rs/pull/19)
- feat(libspot): restore beta.6.0 and expose custom math/float functions by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#18](https://github.com/shenxiangzhuang/libspot-rs/pull/18)
- revert: undo the initial beta.6.0 update by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#17](https://github.com/shenxiangzhuang/libspot-rs/pull/17)
- chore(libspot): bump to beta.6.0 by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#15](https://github.com/shenxiangzhuang/libspot-rs/pull/15)
- docs: simplify README usage examples by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#14](https://github.com/shenxiangzhuang/libspot-rs/pull/14)

## [2.0.0-beta.5.0](https://github.com/shenxiangzhuang/libspot-rs/releases/tag/libspot%28ffi%29-v2.0.0-beta.5.0) - 2025-09-16

- chore(libspot): bump to beta.5.0 by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#13](https://github.com/shenxiangzhuang/libspot-rs/pull/13)
- fix(libspot): use usize in FFI bindings by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#12](https://github.com/shenxiangzhuang/libspot-rs/pull/12)
- docs: remove the obsolete CI badge by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#11](https://github.com/shenxiangzhuang/libspot-rs/pull/11)

## 2.0.0-beta.4.0 (no GitHub Release)

Version recorded in the crate manifest by #10.

- refactor: separate the FFI and pure Rust crates by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) and [**@Copilot**](https://github.com/apps/copilot-swe-agent) in [#10](https://github.com/shenxiangzhuang/libspot-rs/pull/10)
- docs: add badges and installation commands by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#9](https://github.com/shenxiangzhuang/libspot-rs/pull/9)

## [2.0.0-beta.3.0](https://github.com/shenxiangzhuang/libspot-rs/releases/tag/v2.0.0-beta.3.0) - 2025-07-11

- fix(publish): build the C library in OUT_DIR by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#8](https://github.com/shenxiangzhuang/libspot-rs/pull/8)
- build(libspot): link the C library statically by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#7](https://github.com/shenxiangzhuang/libspot-rs/pull/7)
- docs: update benchmark results and commands by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#6](https://github.com/shenxiangzhuang/libspot-rs/pull/6)
- chore(libspot): improve examples and remove unused RNG dependencies by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#5](https://github.com/shenxiangzhuang/libspot-rs/pull/5)
- docs: simplify the README by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#4](https://github.com/shenxiangzhuang/libspot-rs/pull/4)
- docs: improve the quick-start example by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#3](https://github.com/shenxiangzhuang/libspot-rs/pull/3)
- refactor(libspot): modularize the wrapper and add tests and CI by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#2](https://github.com/shenxiangzhuang/libspot-rs/pull/2)
- fix: track the C library as a Git submodule by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [#1](https://github.com/shenxiangzhuang/libspot-rs/pull/1)
- feat(libspot): add the initial C SPOT wrapper by [**@shenxiangzhuang**](https://github.com/shenxiangzhuang) in [8837bd9](https://github.com/shenxiangzhuang/libspot-rs/commit/8837bd9f5086732258431de51eccaa8b39df0d31)
