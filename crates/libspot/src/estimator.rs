//! GPD parameter estimators
//!
//! This module implements Method of Moments (MoM) and Grimshaw estimators
//! for Generalized Pareto Distribution parameters.

use crate::math::{is_nan, xlog, xmin};
use crate::peaks::Peaks;

/// Default epsilon for Brent's method
const BRENT_DEFAULT_EPSILON: f64 = 2.0e-8;

/// Maximum iterations for Brent's method
const BRENT_ITMAX: usize = 200;

/// Method of Moments estimator for GPD parameters
pub fn mom_estimator(peaks: &Peaks) -> (f64, f64, f64) {
    let e = peaks.mean();
    let v = peaks.variance();
    
    if is_nan(e) || is_nan(v) || v <= 0.0 {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
    
    let r = e * e / v;
    let gamma = 0.5 * (1.0 - r);
    let sigma = 0.5 * e * (1.0 + r);
    let log_likelihood = compute_log_likelihood(peaks, gamma, sigma);
    
    (gamma, sigma, log_likelihood)
}

/// Grimshaw estimator for GPD parameters
pub fn grimshaw_estimator(peaks: &Peaks) -> (f64, f64, f64) {
    let mini = peaks.min();
    let maxi = peaks.max();
    let mean = peaks.mean();
    
    if is_nan(mini) || is_nan(maxi) || is_nan(mean) {
        return (f64::NAN, f64::NAN, f64::NAN);
    }
    
    let epsilon = xmin(BRENT_DEFAULT_EPSILON, 0.5 / maxi);
    
    let mut found = [true, false, false]; // true, false, false
    let mut roots = [0.0, 0.0, 0.0]; // 0., ?, ?
    
    // Left root
    let a = -1.0 / maxi + epsilon;
    let b = -epsilon;
    if let Some(root) = brent(a, b, |x| grimshaw_w(x, peaks), BRENT_DEFAULT_EPSILON) {
        roots[1] = root;
        found[1] = true;
    }
    
    // Right root  
    let a = epsilon;
    let b = 2.0 * (mean - mini) / (mini * mini);
    if let Some(root) = brent(a, b, |x| grimshaw_w(x, peaks), BRENT_DEFAULT_EPSILON) {
        roots[2] = root;
        found[2] = true;
    }
    
    // Debug logging for root finding results
    if std::env::var("SPOT_DEBUG_GRIMSHAW").is_ok() {
        println!("Grimshaw roots: found=[{}, {}, {}], roots=[{:.15}, {:.15}, {:.15}]", 
                 found[0], found[1], found[2], roots[0], roots[1], roots[2]);
    }
    
    // Compare all roots (exact C implementation logic)
    let (mut best_gamma, mut best_sigma, mut max_llhood) = 
        grimshaw_simplified_log_likelihood(roots[0], peaks);
    let mut best_root_index = 0;
    
    // Check other roots
    for k in 1..3 {
        if found[k] {
            let (tmp_gamma, tmp_sigma, llhood) = 
                grimshaw_simplified_log_likelihood(roots[k], peaks);
            if llhood > max_llhood {
                max_llhood = llhood;
                best_gamma = tmp_gamma;
                best_sigma = tmp_sigma;
                best_root_index = k;
            }
        }
    }
    
    // Debug logging for final selection
    if std::env::var("SPOT_DEBUG_GRIMSHAW").is_ok() {
        // Only log when there's a change or at key intervals
        println!("Grimshaw selected: root_index={}, gamma={:.15}, sigma={:.15}, llhood={:.15}",
                 best_root_index, best_gamma, best_sigma, max_llhood);
    } else if std::env::var("SPOT_DEBUG_FINAL").is_ok() {
        println!("Final Grimshaw: gamma={:.15}, sigma={:.15}", best_gamma, best_sigma);
    }
    
    (best_gamma, best_sigma, max_llhood)
}

/// Compute log-likelihood for GPD with given parameters
pub fn compute_log_likelihood(peaks: &Peaks, gamma: f64, sigma: f64) -> f64 {
    let nt_local = peaks.size();
    let nt = nt_local as f64;
    
    if nt == 0.0 || sigma <= 0.0 {
        return f64::NEG_INFINITY;
    }
    
    if gamma == 0.0 {
        return -nt * xlog(sigma) - peaks.sum() / sigma;
    }
    
    let mut r = -nt * xlog(sigma);
    let c = 1.0 + 1.0 / gamma;
    let x = gamma / sigma;
    
    // Iterate through container data directly (matches C implementation)
    for i in 0..nt_local {
        if let Some(value) = peaks.container().get(i) {
            let term = 1.0 + x * value;
            if term <= 0.0 {
                return f64::NEG_INFINITY; // Invalid parameters
            }
            r += -c * xlog(term);
        }
    }
    
    r
}

/// Grimshaw w function for root finding
fn grimshaw_w(x: f64, peaks: &Peaks) -> f64 {
    let nt_local = peaks.size();
    let mut u = 0.0;
    let mut v = 0.0;
    
    for i in 0..nt_local {
        if let Some(data_i) = peaks.container().get(i) {
            let s = 1.0 + x * data_i;
            if s <= 0.0 {
                return f64::NAN; // Invalid
            }
            u += 1.0 / s;
            v += xlog(s);
        }
    }
    
    if nt_local == 0 {
        return f64::NAN;
    }
    
    let nt = nt_local as f64;
    (u / nt) * (1.0 + v / nt) - 1.0
}

/// Grimshaw v function
fn grimshaw_v(x: f64, peaks: &Peaks) -> f64 {
    let mut v = 0.0;
    let nt_local = peaks.size();
    
    for i in 0..nt_local {
        if let Some(data_i) = peaks.container().get(i) {
            v += xlog(1.0 + x * data_i);
        }
    }
    
    let nt = nt_local as f64;
    1.0 + v / nt
}

/// Compute simplified log likelihood for Grimshaw method
fn grimshaw_simplified_log_likelihood(x_star: f64, peaks: &Peaks) -> (f64, f64, f64) {
    let (gamma, sigma) = if x_star == 0.0 {
        (0.0, peaks.mean())
    } else {
        let gamma = grimshaw_v(x_star, peaks) - 1.0;
        let sigma = gamma / x_star;
        (gamma, sigma)
    };
    
    let log_likelihood = compute_log_likelihood(peaks, gamma, sigma);
    (gamma, sigma, log_likelihood)
}

/// Brent's method for root finding
/// Returns Some(root) if found, None otherwise
/// This implementation matches the C libspot brent.c exactly
fn brent<F>(x1: f64, x2: f64, func: F, tol: f64) -> Option<f64>
where
    F: Fn(f64) -> f64,
{
    let mut a = x1;
    let mut b = x2;
    let mut c = x2;
    let mut d = 0.0;
    let mut e = 0.0;

    let mut fa = func(a);
    let mut fb = func(b);

    if is_nan(fa) || is_nan(fb) {
        return None;
    }

    // Check that root is bracketed
    if (fa > 0.0 && fb > 0.0) || (fa < 0.0 && fb < 0.0) {
        return None;
    }

    let mut fc = fb;
    
    for _iter in 0..BRENT_ITMAX {
        if (fb > 0.0 && fc > 0.0) || (fb < 0.0 && fc < 0.0) {
            c = a; // Rename a, b, c and adjust bounding interval
            fc = fa;
            e = b - a; // Match C: e = d = b - a
            d = e;
        }
        if fc.abs() < fb.abs() {
            a = b;
            b = c;
            c = a;
            fa = fb;
            fb = fc;
            fc = fa;
        }
        let tol1 = 2.0 * BRENT_DEFAULT_EPSILON * b.abs() + 0.5 * tol; // Convergence check.
        let xm = 0.5 * (c - b);
        if xm.abs() <= tol1 || fb == 0.0 {
            return Some(b);
        }
        if e.abs() >= tol1 && fa.abs() > fb.abs() {
            let s = fb / fa; // Attempt inverse quadratic interpolation.
            let (p, q) = if a == c {
                let p = 2.0 * xm * s;
                let q = 1.0 - s;
                (p, q)
            } else {
                let q = fa / fc;
                let r = fb / fc;
                let p = s * (2.0 * xm * q * (q - r) - (b - a) * (r - 1.0));
                let q = (q - 1.0) * (r - 1.0) * (s - 1.0);
                (p, q)
            };
            
            let q = if p > 0.0 {
                -q // Check whether in bounds.
            } else {
                q
            };
            let p = p.abs();
            
            let min1 = 3.0 * xm * q - (tol1 * q).abs();
            let min2 = (e * q).abs();
            if 2.0 * p < if min1 < min2 { min1 } else { min2 } {
                e = d; // Accept interpolation.
                d = p / q;
            } else {
                d = xm; // Interpolation failed, use bisection.
                e = d;
            }
        } else { // Bounds decreasing too slowly, use bisection.
            d = xm;
            e = d;
        }
        a = b; // Move last best guess to a.
        fa = fb;
        if d.abs() > tol1 {
            // Evaluate new trial root.
            b += d;
        } else {
            b += if xm >= 0.0 { tol1.abs() } else { -tol1.abs() };
        }
        fb = func(b);
        if is_nan(fb) {
            return None;
        }
    }
    // Maximum number of iterations exceeded
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peaks::Peaks;
    use approx::assert_relative_eq;

    #[test]
    fn test_mom_estimator_empty_peaks() {
        let peaks = Peaks::new(5).unwrap();
        let (gamma, sigma, llhood) = mom_estimator(&peaks);
        assert!(is_nan(gamma));
        assert!(is_nan(sigma));
        assert!(is_nan(llhood));
    }

    #[test]
    fn test_mom_estimator_single_value() {
        let mut peaks = Peaks::new(5).unwrap();
        peaks.push(1.0);
        
        let (gamma, sigma, _llhood) = mom_estimator(&peaks);
        // With variance = 0, this should produce specific values
        assert!(is_nan(gamma) || gamma.is_infinite());
        assert!(is_nan(sigma) || sigma.is_infinite());
    }

    #[test]
    fn test_mom_estimator_normal_case() {
        let mut peaks = Peaks::new(10).unwrap();
        for value in [1.0, 2.0, 3.0, 4.0, 5.0] {
            peaks.push(value);
        }
        
        let (gamma, sigma, llhood) = mom_estimator(&peaks);
        assert!(!is_nan(gamma));
        assert!(!is_nan(sigma));
        assert!(!is_nan(llhood));
        assert!(sigma > 0.0); // Sigma should be positive
    }

    #[test]
    fn test_log_likelihood_gamma_zero() {
        let mut peaks = Peaks::new(10).unwrap();
        peaks.push(1.0);
        peaks.push(2.0);
        peaks.push(3.0);
        
        let ll = compute_log_likelihood(&peaks, 0.0, 2.0);
        assert!(!is_nan(ll));
        assert!(ll.is_finite());
    }

    #[test]
    fn test_log_likelihood_gamma_nonzero() {
        let mut peaks = Peaks::new(10).unwrap();
        peaks.push(1.0);
        peaks.push(2.0);
        peaks.push(3.0);
        
        let ll = compute_log_likelihood(&peaks, 0.1, 2.0);
        assert!(!is_nan(ll));
        assert!(ll.is_finite());
    }

    #[test]
    fn test_brent_simple_function() {
        // Find root of x^2 - 4 = 0 in [1, 3], should find x = 2
        let result = brent(1.0, 3.0, |x| x * x - 4.0, 1e-10);
        assert!(result.is_some());
        let root = result.unwrap();
        assert_relative_eq!(root, 2.0, epsilon = 1e-9);
    }

    #[test]
    fn test_brent_no_root() {
        // Function x^2 + 1 has no real roots
        let result = brent(-1.0, 1.0, |x| x * x + 1.0, 1e-10);
        assert!(result.is_none());
    }
}