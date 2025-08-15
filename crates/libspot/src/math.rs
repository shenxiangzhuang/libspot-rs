//! Mathematical functions that replicate the xmath.c implementation
//!
//! This module provides the core mathematical functions used by the SPOT algorithm,
//! implemented in pure Rust to match the C behavior exactly.

/// Constant for LOG(2) - exact same hex representation as C implementation
const LOG2: f64 = f64::from_bits(0x3FE62E42FEFA39EF);

/// Check if a double is NaN
#[inline]
pub fn is_nan(x: f64) -> bool {
    x != x
}

/// Return the minimum of two values
#[inline]
pub fn xmin(a: f64, b: f64) -> f64 {
    if is_nan(a) || is_nan(b) {
        f64::NAN
    } else if a < b {
        a
    } else {
        b
    }
}

/// Natural logarithm using Shanks' continued fraction algorithm
/// Returns -∞ for x=0 and NaN for x<0
pub fn xlog(x: f64) -> f64 {
    if x < 0.0 || is_nan(x) {
        return f64::NAN;
    }
    if x == 0.0 {
        return f64::NEG_INFINITY;
    }

    // Use frexp to extract mantissa and exponent
    let (mantissa, exponent) = extract_frexp(x);
    
    if exponent == 0 || exponent == -1 {
        return log_cf_11(x);
    }
    
    log_cf_11(mantissa) + LOG2 * (exponent as f64)
}

/// Exponential function using Khovanskii's continued fraction
pub fn xexp(x: f64) -> f64 {
    if is_nan(x) {
        return f64::NAN;
    }
    if x < 0.0 {
        return 1.0 / xexp(-x);
    }
    if x > LOG2 {
        let k = (x / LOG2) as u32;
        let r = x - LOG2 * (k as f64);
        return exp_cf_6(r) * (2.0_f64).powi(k as i32);
    }

    exp_cf_6(x)
}

/// Power function: a^x = exp(x * ln(a))
pub fn xpow(a: f64, x: f64) -> f64 {
    xexp(x * xlog(a))
}

/// Logarithm continued fraction implementation (11th order)
fn log_cf_11(z: f64) -> f64 {
    let x = z - 1.0;
    let xx = x + 2.0;
    let x2 = x * x;

    let xx2 = xx + xx;
    let xx3 = xx + xx2;
    let xx5 = xx3 + xx2;
    let xx7 = xx5 + xx2;
    let xx9 = xx7 + xx2;
    let xx11 = xx9 + xx2;
    let xx13 = xx11 + xx2;
    let xx15 = xx13 + xx2;
    let xx17 = xx15 + xx2;
    let xx19 = xx17 + xx2;
    let xx21 = xx19 + xx2;

    2.0 * x /
        (-x2 / (-4.0 * x2 /
                   (-9.0 * x2 /
                        (-16.0 * x2 /
                             (-25.0 * x2 /
                                  (-36.0 * x2 /
                                       (-49.0 * x2 /
                                            (-64.0 * x2 /
                                                 (-81.0 * x2 /
                                                      (-100.0 * x2 / xx21 +
                                                       xx19) +
                                                  xx17) +
                                             xx15) +
                                        xx13) +
                                   xx11) +
                              xx9) +
                         xx7) +
                    xx5) +
               xx3) +
        xx)
}

/// Exponential continued fraction implementation (6th order)
fn exp_cf_6(z: f64) -> f64 {
    let z2 = z * z;

    2.0 * z /
           (2.0 * z2 /
                (12.0 * z2 /
                     (60.0 * z2 / (140.0 * z2 / (7.0 * z2 / 11.0 + 252.0) + 140.0) +
                      60.0) +
                 12.0) -
            z + 2.0) +
       1.0
}

/// Extract mantissa and exponent from floating point number
/// Replicates the behavior of frexp()
fn extract_frexp(x: f64) -> (f64, i32) {
    if x == 0.0 {
        return (x, 0);
    }
    
    let bits = x.to_bits();
    let sign = if bits & (1u64 << 63) != 0 { -1.0 } else { 1.0 };
    let exp_bits = (bits >> 52) & 0x7ff;
    let mantissa_bits = bits & 0xfffffffffffff;
    
    if exp_bits == 0 {
        // Subnormal number
        if x != 0.0 {
            let (norm_mantissa, norm_exp) = extract_frexp(x * (1u64 << 52) as f64);
            return (norm_mantissa, norm_exp - 52);
        } else {
            return (x, 0);
        }
    } else if exp_bits == 0x7ff {
        // Infinity or NaN
        return (x, 0);
    }
    
    let exponent = exp_bits as i32 - 0x3fe;
    let mantissa = sign * f64::from_bits(mantissa_bits | 0x3fe0000000000000);
    
    (mantissa, exponent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_is_nan() {
        assert!(is_nan(f64::NAN));
        assert!(!is_nan(1.0));
        assert!(!is_nan(0.0));
        assert!(!is_nan(f64::INFINITY));
    }

    #[test]
    fn test_xmin() {
        assert_relative_eq!(xmin(1.0, 2.0), 1.0);
        assert_relative_eq!(xmin(2.0, 1.0), 1.0);
        assert!(is_nan(xmin(f64::NAN, 1.0)));
        assert!(is_nan(xmin(1.0, f64::NAN)));
    }

    #[test]
    fn test_xlog() {
        assert_relative_eq!(xlog(1.0), 0.0, epsilon = 1e-15);
        assert_relative_eq!(xlog(std::f64::consts::E), 1.0, epsilon = 1e-14);
        assert_relative_eq!(xlog(2.0), LOG2, epsilon = 1e-15);
        assert!(is_nan(xlog(-1.0)));
        assert_eq!(xlog(0.0), f64::NEG_INFINITY);
    }

    #[test]
    fn test_xexp() {
        assert_relative_eq!(xexp(0.0), 1.0, epsilon = 1e-15);
        assert_relative_eq!(xexp(1.0), std::f64::consts::E, epsilon = 1e-14);
        assert_relative_eq!(xexp(LOG2), 2.0, epsilon = 1e-14);
    }

    #[test]
    fn test_xpow() {
        assert_relative_eq!(xpow(2.0, 3.0), 8.0, epsilon = 1e-14);
        assert_relative_eq!(xpow(std::f64::consts::E, 2.0), std::f64::consts::E * std::f64::consts::E, epsilon = 1e-13);
        assert_relative_eq!(xpow(4.0, 0.5), 2.0, epsilon = 1e-14);
    }

    #[test]
    fn test_frexp() {
        let (mantissa, exp) = extract_frexp(8.0);
        assert_relative_eq!(mantissa, 0.5, epsilon = 1e-15);
        assert_eq!(exp, 4);
        
        let (mantissa, exp) = extract_frexp(0.5);
        assert_relative_eq!(mantissa, 0.5, epsilon = 1e-15);
        assert_eq!(exp, 0);
    }
}