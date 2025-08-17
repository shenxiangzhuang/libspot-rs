//! Isolated Rust implementation of the Grimshaw estimator
//! This file extracts the core Grimshaw estimator logic for debugging

use std::fs::File;
use std::io::{BufRead, BufReader};

// Copy of Float type
type Float = f64;

// Copy of mathematical functions
fn is_nan(x: Float) -> bool {
    x.is_nan()
}

fn xlog(x: Float) -> Float {
    x.ln()
}

fn xmin(a: Float, b: Float) -> Float {
    a.min(b)
}

/// Default epsilon for Brent's method
const BRENT_DEFAULT_EPSILON: Float = 2.0e-8;

/// Maximum iterations for Brent's method
const BRENT_ITMAX: usize = 200;

/// Simplified peaks data structure for testing
pub struct SimplePeaks {
    data: Vec<Float>,
}

impl SimplePeaks {
    pub fn new(data: Vec<Float>) -> Self {
        Self { data }
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn mean(&self) -> Float {
        if self.data.is_empty() {
            return Float::NAN;
        }
        self.data.iter().sum::<Float>() / (self.data.len() as Float)
    }

    pub fn min(&self) -> Float {
        self.data.iter().fold(Float::NAN, |a, &b| if a.is_nan() || b < a { b } else { a })
    }

    pub fn max(&self) -> Float {
        self.data.iter().fold(Float::NAN, |a, &b| if a.is_nan() || b > a { b } else { a })
    }

    pub fn sum(&self) -> Float {
        self.data.iter().sum()
    }

    pub fn get(&self, index: usize) -> Option<Float> {
        self.data.get(index).copied()
    }
}

/// Grimshaw estimator for GPD parameters
pub fn grimshaw_estimator(peaks: &SimplePeaks) -> (Float, Float, Float) {
    let mini = peaks.min();
    let maxi = peaks.max();
    let mean = peaks.mean();
    
    if is_nan(mini) || is_nan(maxi) || is_nan(mean) {
        return (Float::NAN, Float::NAN, Float::NAN);
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
    
    // Compare all roots (exact C implementation logic)
    let (mut best_gamma, mut best_sigma, mut max_llhood) = 
        grimshaw_simplified_log_likelihood(roots[0], peaks);
    
    // Check other roots
    for k in 1..3 {
        if found[k] {
            let (tmp_gamma, tmp_sigma, llhood) = 
                grimshaw_simplified_log_likelihood(roots[k], peaks);
            if llhood > max_llhood {
                max_llhood = llhood;
                best_gamma = tmp_gamma;
                best_sigma = tmp_sigma;
            }
        }
    }
    
    (best_gamma, best_sigma, max_llhood)
}

/// Compute log-likelihood for GPD with given parameters
pub fn compute_log_likelihood(peaks: &SimplePeaks, gamma: Float, sigma: Float) -> Float {
    let nt_local = peaks.size();
    let nt = nt_local as f64;
    
    if nt == 0.0 || sigma <= 0.0 {
        return Float::NEG_INFINITY;
    }
    
    if gamma == 0.0 {
        return -nt * xlog(sigma) - peaks.sum() / sigma;
    }
    
    let mut r = -nt * xlog(sigma);
    let c = 1.0 + 1.0 / gamma;
    let x = gamma / sigma;
    
    // Iterate through container data directly (matches C implementation)
    for i in 0..nt_local {
        if let Some(value) = peaks.get(i) {
            let term = 1.0 + x * value;
            if term <= 0.0 {
                return Float::NEG_INFINITY; // Invalid parameters
            }
            r += -c * xlog(term);
        }
    }
    
    r
}

/// Grimshaw w function for root finding
fn grimshaw_w(x: Float, peaks: &SimplePeaks) -> Float {
    let nt_local = peaks.size();
    let mut u = 0.0;
    let mut v = 0.0;
    
    for i in 0..nt_local {
        if let Some(data_i) = peaks.get(i) {
            let s = 1.0 + x * data_i;
            if s <= 0.0 {
                return Float::NAN; // Invalid
            }
            u += 1.0 / s;
            v += xlog(s);
        }
    }
    
    if nt_local == 0 {
        return Float::NAN;
    }
    
    let nt = nt_local as f64;
    (u / nt) * (1.0 + v / nt) - 1.0
}

/// Grimshaw v function
fn grimshaw_v(x: Float, peaks: &SimplePeaks) -> Float {
    let mut v = 0.0;
    let nt_local = peaks.size();
    
    for i in 0..nt_local {
        if let Some(data_i) = peaks.get(i) {
            v += xlog(1.0 + x * data_i);
        }
    }
    
    let nt = nt_local as f64;
    1.0 + v / nt
}

/// Compute simplified log likelihood for Grimshaw method
fn grimshaw_simplified_log_likelihood(x_star: Float, peaks: &SimplePeaks) -> (Float, Float, Float) {
    // Use a tolerance to handle roots that are very close to zero (matching C behavior)
    let (gamma, sigma) = if x_star.abs() <= BRENT_DEFAULT_EPSILON || x_star == 0.0 {
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
fn brent<F>(x1: Float, x2: Float, func: F, tol: Float) -> Option<Float>
where
    F: Fn(Float) -> Float,
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
            d = b - a; // Match C exactly: e = d = b - a
            e = d;
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

/// Load peaks data from CSV file
fn load_peaks_data(filename: &str) -> Result<SimplePeaks, Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let mut data = Vec::new();
    
    for line in reader.lines() {
        let line = line?;
        if line.starts_with('#') || line.starts_with("index") {
            continue;
        }
        if let Some(comma_pos) = line.find(',') {
            let value_str = &line[comma_pos + 1..];
            if let Ok(value) = value_str.parse::<Float>() {
                data.push(value);
            }
        }
    }
    
    Ok(SimplePeaks::new(data))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing isolated Rust Grimshaw estimator");
    
    // Load the critical peaks data
    let peaks = load_peaks_data("critical_peaks_data.csv")?;
    println!("Loaded {} peaks values", peaks.size());
    println!("Peaks mean: {:.15}", peaks.mean());
    println!("Peaks min: {:.15}", peaks.min());
    println!("Peaks max: {:.15}", peaks.max());
    
    // Run the Grimshaw estimator
    let (gamma, sigma, llhood) = grimshaw_estimator(&peaks);
    println!("Rust Grimshaw Results:");
    println!("Gamma: {:.15}", gamma);
    println!("Sigma: {:.15}", sigma);
    println!("Log-likelihood: {:.15}", llhood);
    
    // Compute the anomaly threshold like SPOT does
    let q = 0.0001;
    let nt = 166; // Number of Z updates
    let n = 97066; // Total number of seen data
    let excess_threshold = 6.236165177550786;
    let up_down = 1.0; // For upper tail
    
    let s = (nt as Float) / (n as Float);
    let r = q / s;
    
    // Compute tail quantile
    let tail_quantile = if gamma == 0.0 {
        -sigma * r.ln()
    } else {
        (sigma / gamma) * (r.powf(-gamma) - 1.0)
    };
    
    let z = excess_threshold + up_down * tail_quantile;
    
    println!("Anomaly threshold Z: {:.15}", z);
    println!("Tail quantile: {:.15}", tail_quantile);
    println!("s (nt/n): {:.15}", s);
    println!("r (q/s): {:.15}", r);
    
    Ok(())
}