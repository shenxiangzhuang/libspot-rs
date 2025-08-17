# Grimshaw Estimator Debugging Experiment

This directory contains isolated testing and debugging tools to investigate numerical precision differences between the pure Rust and C FFI implementations of the SPOT algorithm's Grimshaw estimator.

## Key Finding

**The Grimshaw estimator implementations are mathematically equivalent** - both pure Rust and C FFI produce nearly identical results (differences only in the 15th decimal place). The real divergence occurs elsewhere in the SPOT algorithm integration.

## Files

### Data Files
- `critical_peaks_data.csv` - Exact peaks data captured at step 97066 (166th Z update) where divergence occurs
- `critical_input.txt` - The critical input value (X=7.099368295593661) that triggers the divergence
- `comparison_report.txt` - Detailed comparison report of both implementations

### Source Files
- `capture_critical_data.rs` - Captures the exact peaks data at the critical divergence point
- `rust_grimshaw.rs` - Isolated pure Rust implementation of Grimshaw estimator
- `c_grimshaw.rs` - Isolated C FFI implementation calling the C library directly
- `comprehensive_comparison.rs` - Side-by-side comparison of both implementations

### Build Files
- `Cargo.toml` - Project configuration with all binaries
- `build.rs` - Links with the C libspot library

## Results Summary

### Critical Point Analysis
- **Divergence Point**: Step 97066 (166th Z update)
- **Input Value**: X = 7.099368295593661
- **Full SPOT Results**:
  - Pure Rust: Z = 8.285563050271193
  - C FFI: Z = 8.272309469981852 (expected correct value)

### Isolated Grimshaw Estimator Results
Both isolated implementations produce nearly identical results:

- **Pure Rust**: Z = 8.269205530513512
- **C FFI**: Z = 8.269205530513515
- **Difference**: 3.55e-15 (negligible)

### Key Insight
The isolated estimators both produce Z ≈ 8.2692, but the full SPOT implementations produce different values (8.2856 vs 8.2723). This proves:

1. ✅ **Grimshaw estimator is correct** - Both implementations are mathematically equivalent
2. ❌ **Issue is in SPOT integration** - The problem is in how the estimator is called or how its results are used
3. 🎯 **Root cause isolated** - The divergence is NOT in the optimization algorithm itself

## Running the Tests

```bash
# Capture critical data from full SPOT run
cargo run --bin capture_critical_data

# Test isolated Rust implementation
cargo run --bin rust_grimshaw

# Test isolated C FFI implementation  
cargo run --bin c_grimshaw

# Run comprehensive comparison
cargo run --bin comprehensive_comparison
```

## Next Steps

The debugging should now focus on:

1. **Parameter differences** - Are nt, n, or other parameters different between implementations?
2. **Data handling** - Is the peaks data being processed differently in the full algorithm?
3. **Integration points** - Where exactly in the SPOT algorithm do the implementations diverge?

The Grimshaw estimator itself is mathematically correct and equivalent between implementations.