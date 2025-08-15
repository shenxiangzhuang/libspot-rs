# libspot-ffi

Rust FFI bindings for the [libspot](https://github.com/asiffer/libspot) C library, a fast time series anomaly detector based on the SPOT (Streaming Peaks Over Threshold) algorithm.

## Overview

This crate provides Rust bindings to the original C implementation of libspot. It uses Foreign Function Interface (FFI) to call the C library functions, providing a Rust-friendly API while leveraging the performance of the optimized C implementation.

## Features

- **C Library Integration**: Direct bindings to the libspot C library
- **Memory Management**: Safe Rust wrappers around C memory management
- **Type Safety**: Rust type system ensures safe usage of C functions
- **Error Handling**: Rust Result types for error management

## Usage

```rust
use libspot_ffi::{SpotDetector, SpotConfig, SpotStatus};

let config = SpotConfig::default();
let mut detector = SpotDetector::new(config)?;

detector.fit(&training_data)?;
match detector.step(value)? {
    SpotStatus::Normal => println!("Normal"),
    SpotStatus::Excess => println!("In tail"),
    SpotStatus::Anomaly => println!("Anomaly detected!"),
}
```

## Requirements

- C compiler (gcc, clang, etc.)
- libspot C library (included as git submodule)

## Alternative

For a pure Rust implementation without FFI dependencies, see the [`libspot`](../libspot) crate.