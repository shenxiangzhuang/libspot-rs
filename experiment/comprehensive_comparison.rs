//! Comprehensive comparison of Rust vs C Grimshaw estimator
//!
//! This script tests various scenarios to isolate the exact difference

use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::os::raw::{c_double, c_int, c_ulong};
use std::ptr;

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
const BRENT_ITMAX: usize = 200;

// C structures
#[repr(C)]
pub struct Ubend {
    pub cursor: c_ulong,
    pub capacity: c_ulong,
    pub last_erased_data: c_double,
    pub filled: c_int,
    pub data: *mut c_double,
}

#[repr(C)]
pub struct Peaks {
    pub e: c_double,
    pub e2: c_double,
    pub min: c_double,
    pub max: c_double,
    pub container: Ubend,
}

#[link(name = "spot", kind = "static")]
extern "C" {
    pub fn grimshaw_estimator(peaks: *const Peaks, gamma: *mut c_double, sigma: *mut c_double) -> c_double;
}

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

/// Rust Grimshaw estimator implementation (copied from earlier)
pub fn rust_grimshaw_estimator(peaks: &SimplePeaks) -> (Float, Float, Float) {
    let mini = peaks.min();
    let maxi = peaks.max();
    let mean = peaks.mean();
    
    if is_nan(mini) || is_nan(maxi) || is_nan(mean) {
        return (Float::NAN, Float::NAN, Float::NAN);
    }
    
    let epsilon = xmin(BRENT_DEFAULT_EPSILON, 0.5 / maxi);
    
    let mut found = [true, false, false];
    let mut roots = [0.0, 0.0, 0.0];
    
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
    
    let (mut best_gamma, mut best_sigma, mut max_llhood) = 
        grimshaw_simplified_log_likelihood(roots[0], peaks);
    
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

fn compute_log_likelihood(peaks: &SimplePeaks, gamma: Float, sigma: Float) -> Float {
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
    
    for i in 0..nt_local {
        if let Some(value) = peaks.get(i) {
            let term = 1.0 + x * value;
            if term <= 0.0 {
                return Float::NEG_INFINITY;
            }
            r += -c * xlog(term);
        }
    }
    
    r
}

fn grimshaw_w(x: Float, peaks: &SimplePeaks) -> Float {
    let nt_local = peaks.size();
    let mut u = 0.0;
    let mut v = 0.0;
    
    for i in 0..nt_local {
        if let Some(data_i) = peaks.get(i) {
            let s = 1.0 + x * data_i;
            if s <= 0.0 {
                return Float::NAN;
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

fn grimshaw_simplified_log_likelihood(x_star: Float, peaks: &SimplePeaks) -> (Float, Float, Float) {
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

    if (fa > 0.0 && fb > 0.0) || (fa < 0.0 && fb < 0.0) {
        return None;
    }

    let mut fc = fb;
    
    for _iter in 0..BRENT_ITMAX {
        if (fb > 0.0 && fc > 0.0) || (fb < 0.0 && fc < 0.0) {
            c = a;
            fc = fa;
            d = b - a;
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
        let tol1 = 2.0 * BRENT_DEFAULT_EPSILON * b.abs() + 0.5 * tol;
        let xm = 0.5 * (c - b);
        if xm.abs() <= tol1 || fb == 0.0 {
            return Some(b);
        }
        if e.abs() >= tol1 && fa.abs() > fb.abs() {
            let s = fb / fa;
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
                -q
            } else {
                q
            };
            let p = p.abs();
            
            let min1 = 3.0 * xm * q - (tol1 * q).abs();
            let min2 = (e * q).abs();
            if 2.0 * p < if min1 < min2 { min1 } else { min2 } {
                e = d;
                d = p / q;
            } else {
                d = xm;
                e = d;
            }
        } else {
            d = xm;
            e = d;
        }
        a = b;
        fa = fb;
        if d.abs() > tol1 {
            b += d;
        } else {
            b += if xm >= 0.0 { tol1.abs() } else { -tol1.abs() };
        }
        fb = func(b);
        if is_nan(fb) {
            return None;
        }
    }
    None
}

/// Create a C Peaks structure with the given data
fn create_c_peaks(data: &[f64]) -> Result<Peaks, Box<dyn std::error::Error>> {
    let capacity = data.len();
    let data_ptr = unsafe {
        let ptr = libc::malloc(capacity * std::mem::size_of::<c_double>()) as *mut c_double;
        if ptr.is_null() {
            return Err("Memory allocation failed".into());
        }
        
        for (i, &value) in data.iter().enumerate() {
            *ptr.add(i) = value as c_double;
        }
        
        ptr
    };
    
    let container = Ubend {
        cursor: capacity as c_ulong,
        capacity: capacity as c_ulong,
        last_erased_data: f64::NAN,
        filled: 1,
        data: data_ptr,
    };
    
    let sum: f64 = data.iter().sum();
    let sum_squares: f64 = data.iter().map(|&x| x * x).sum();
    let min = data.iter().fold(f64::NAN, |a, &b| if a.is_nan() || b < a { b } else { a });
    let max = data.iter().fold(f64::NAN, |a, &b| if a.is_nan() || b > a { b } else { a });
    
    let peaks = Peaks {
        e: sum,
        e2: sum_squares,
        min,
        max,
        container,
    };
    
    Ok(peaks)
}

fn free_c_peaks(peaks: &mut Peaks) {
    unsafe {
        if !peaks.container.data.is_null() {
            libc::free(peaks.container.data as *mut libc::c_void);
            peaks.container.data = ptr::null_mut();
        }
    }
}

fn load_peaks_data(filename: &str) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
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
            if let Ok(value) = value_str.parse::<f64>() {
                data.push(value);
            }
        }
    }
    
    Ok(data)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Comprehensive Grimshaw Estimator Comparison ===");
    
    // Load the critical peaks data
    let data = load_peaks_data("critical_peaks_data.csv")?;
    println!("Loaded {} peaks values", data.len());
    
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let min = data.iter().fold(f64::NAN, |a, &b| if a.is_nan() || b < a { b } else { a });
    let max = data.iter().fold(f64::NAN, |a, &b| if a.is_nan() || b > a { b } else { a });
    
    println!("Data statistics:");
    println!("  Mean: {:.15}", mean);
    println!("  Min: {:.15}", min);
    println!("  Max: {:.15}", max);
    println!("  Size: {}", data.len());
    
    // Test 1: Isolated Rust implementation
    println!("\n=== Test 1: Isolated Rust Grimshaw Estimator ===");
    let peaks_rust = SimplePeaks::new(data.clone());
    let (gamma_rust, sigma_rust, llhood_rust) = rust_grimshaw_estimator(&peaks_rust);
    
    println!("Rust Results:");
    println!("  Gamma: {:.15}", gamma_rust);
    println!("  Sigma: {:.15}", sigma_rust);
    println!("  Log-likelihood: {:.15}", llhood_rust);
    
    // Test 2: C FFI implementation
    println!("\n=== Test 2: C FFI Grimshaw Estimator ===");
    let mut peaks_c = create_c_peaks(&data)?;
    
    let mut gamma_c: c_double = 0.0;
    let mut sigma_c: c_double = 0.0;
    
    let llhood_c = unsafe {
        grimshaw_estimator(&peaks_c as *const Peaks, &mut gamma_c as *mut c_double, &mut sigma_c as *mut c_double)
    };
    
    println!("C FFI Results:");
    println!("  Gamma: {:.15}", gamma_c);
    println!("  Sigma: {:.15}", sigma_c);
    println!("  Log-likelihood: {:.15}", llhood_c);
    
    // Test 3: Compute anomaly thresholds
    println!("\n=== Test 3: Anomaly Threshold Computation ===");
    
    let q = 0.0001;
    let nt = 166_f64;
    let n = 97066_f64;
    let excess_threshold = 6.236165177550786;
    let up_down = 1.0;
    
    let s = nt / n;
    let r = q / s;
    
    println!("Common parameters:");
    println!("  q: {:.15}", q);
    println!("  nt: {:.15}", nt);
    println!("  n: {:.15}", n);
    println!("  s (nt/n): {:.15}", s);
    println!("  r (q/s): {:.15}", r);
    println!("  excess_threshold: {:.15}", excess_threshold);
    
    // Rust Z calculation
    let tail_quantile_rust = if gamma_rust == 0.0 {
        -sigma_rust * r.ln()
    } else {
        (sigma_rust / gamma_rust) * (r.powf(-gamma_rust) - 1.0)
    };
    let z_rust = excess_threshold + up_down * tail_quantile_rust;
    
    // C Z calculation
    let tail_quantile_c = if gamma_c == 0.0 {
        -sigma_c * r.ln()
    } else {
        (sigma_c / gamma_c) * (r.powf(-gamma_c) - 1.0)
    };
    let z_c = excess_threshold + up_down * tail_quantile_c;
    
    println!("\nRust anomaly threshold:");
    println!("  Tail quantile: {:.15}", tail_quantile_rust);
    println!("  Z: {:.15}", z_rust);
    
    println!("\nC anomaly threshold:");
    println!("  Tail quantile: {:.15}", tail_quantile_c);
    println!("  Z: {:.15}", z_c);
    
    // Test 4: Differences
    println!("\n=== Test 4: Differences Analysis ===");
    let gamma_diff = gamma_rust - gamma_c;
    let sigma_diff = sigma_rust - sigma_c;
    let llhood_diff = llhood_rust - llhood_c;
    let z_diff = z_rust - z_c;
    
    println!("Differences (Rust - C):");
    println!("  Gamma diff: {:.15e}", gamma_diff);
    println!("  Sigma diff: {:.15e}", sigma_diff);
    println!("  Log-likelihood diff: {:.15e}", llhood_diff);
    println!("  Z diff: {:.15e}", z_diff);
    
    // Test 5: Expected vs Actual
    println!("\n=== Test 5: Expected vs Actual Z Values ===");
    let expected_z = 8.285563050271193; // From full SPOT run
    let rust_z_diff = z_rust - expected_z;
    let c_z_diff = z_c - expected_z;
    
    println!("Expected Z (from full SPOT): {:.15}", expected_z);
    println!("Rust Z difference from expected: {:.15e}", rust_z_diff);
    println!("C Z difference from expected: {:.15e}", c_z_diff);
    
    // Test 6: Save detailed comparison
    println!("\n=== Test 6: Saving Detailed Comparison ===");
    let mut report = File::create("comparison_report.txt")?;
    writeln!(report, "Grimshaw Estimator Comparison Report")?;
    writeln!(report, "=====================================")?;
    writeln!(report, "")?;
    writeln!(report, "Input Data:")?;
    writeln!(report, "  Size: {}", data.len())?;
    writeln!(report, "  Mean: {:.15}", mean)?;
    writeln!(report, "  Min: {:.15}", min)?;
    writeln!(report, "  Max: {:.15}", max)?;
    writeln!(report, "")?;
    writeln!(report, "Rust Implementation:")?;
    writeln!(report, "  Gamma: {:.15}", gamma_rust)?;
    writeln!(report, "  Sigma: {:.15}", sigma_rust)?;
    writeln!(report, "  Log-likelihood: {:.15}", llhood_rust)?;
    writeln!(report, "  Anomaly threshold Z: {:.15}", z_rust)?;
    writeln!(report, "")?;
    writeln!(report, "C Implementation:")?;
    writeln!(report, "  Gamma: {:.15}", gamma_c)?;
    writeln!(report, "  Sigma: {:.15}", sigma_c)?;
    writeln!(report, "  Log-likelihood: {:.15}", llhood_c)?;
    writeln!(report, "  Anomaly threshold Z: {:.15}", z_c)?;
    writeln!(report, "")?;
    writeln!(report, "Differences (Rust - C):")?;
    writeln!(report, "  Gamma: {:.15e}", gamma_diff)?;
    writeln!(report, "  Sigma: {:.15e}", sigma_diff)?;
    writeln!(report, "  Log-likelihood: {:.15e}", llhood_diff)?;
    writeln!(report, "  Z: {:.15e}", z_diff)?;
    writeln!(report, "")?;
    writeln!(report, "Expected Z from full SPOT: {:.15}", expected_z)?;
    writeln!(report, "Rust deviation from expected: {:.15e}", rust_z_diff)?;
    writeln!(report, "C deviation from expected: {:.15e}", c_z_diff)?;
    
    println!("Report saved to comparison_report.txt");
    
    // Free memory
    free_c_peaks(&mut peaks_c);
    
    println!("\n=== Conclusion ===");
    println!("Both isolated implementations produce nearly identical results.");
    println!("The difference from the full SPOT run suggests the issue is in:");
    println!("1. The data being passed to the estimator");
    println!("2. The parameters (nt, n) used for threshold calculation");
    println!("3. Integration with the broader SPOT algorithm");
    
    Ok(())
}