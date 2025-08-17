# SPOT Algorithm Debugging Investigation

## Summary

Successfully isolated the root cause of numerical differences between pure Rust and C FFI implementations using systematic component isolation.

## Key Findings

### 1. **Grimshaw Estimator is Mathematically Equivalent** ✅
- Pure Rust: Z = 8.269205530513512
- C FFI: Z = 8.269205530513515  
- Difference: 3.55e-15 (negligible floating-point precision)

### 2. **Data Counting is Identical** ✅
- Both implementations count anomalies, excesses, and normal values identically
- Internal counters (n, nt) match exactly
- Status classifications are consistent

### 3. **Excess Thresholds are Identical** ✅
- Both use same P2 quantile calculation
- Excess threshold differences = 0.00e0

### 4. **Divergence Occurs at First Excess** ❌
- **Critical Point**: Step 1028 (first excess after training)
- **Input**: X = 0.998924517538399
- **Result**: Rust Z = 1.003213786138980, FFI Z = 1.003213797524437
- **Difference**: -1.14e-8

### 5. **Root Cause: Quantile Calculation Precision** ❌
- Systematic numerical differences across multiple quantile values:
  - q=0.0001: diff = -1.14e-8
  - q=0.001:  diff = -6.53e-9
  - q=0.01:   diff = -1.68e-9
  - q=0.1:    diff = 3.17e-9

## Investigation Methodology

Used systematic component isolation approach:

1. **debug_counting.rs** - Verified identical data point classification
2. **find_z_divergence.rs** - Located exact divergence at step 1028
3. **isolate_first_excess.rs** - Analyzed critical first excess case
4. **test_quantile_precision.rs** - Isolated mathematical computation differences

## Technical Conclusion

Both implementations are **mathematically correct** but use different floating-point precision in their calculations. The pure Rust version uses Rust's standard mathematical functions while the C FFI uses the underlying C library's math functions, leading to tiny but systematic differences that compound over time.

For production use requiring **exact** C compatibility, the differences would need to be eliminated by using identical mathematical function implementations.

## Experiment Files

- **comprehensive_comparison.rs** - Proves Grimshaw estimator equivalence
- **critical_peaks_data.csv** - Captured peaks data at divergence point
- **comparison_report.txt** - Detailed mathematical comparison
- All isolation experiments in `experiment/` directory