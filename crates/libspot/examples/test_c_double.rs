//! Test: Grimshaw estimator using c_double to match C library precision
//!
//! This creates a version of the Grimshaw estimator that uses c_double throughout
//! to test if this eliminates the precision differences with the C implementation.

use std::os::raw::c_double;
use libspot::{Spot, SpotConfig, SpotStatus};

extern "C" {
    fn srand(seed: u32);
    fn rand() -> i32;
}

fn c_rand() -> c_double {
    unsafe { rand() as c_double / (i32::MAX as c_double + 1.0) }
}

// Constants using c_double
const BRENT_DEFAULT_EPSILON_CD: c_double = 2.0e-8;
const BRENT_ITMAX: usize = 200;

// Helper functions using c_double
fn xmin_cd(a: c_double, b: c_double) -> c_double {
    if a.is_nan() || b.is_nan() {
        return c_double::NAN;
    }
    if a < b { a } else { b }
}

fn is_nan_cd(x: c_double) -> bool {
    x.is_nan()
}

// Statistics structure using c_double
#[derive(Debug, Clone)]
struct PeaksCD {
    data: Vec<c_double>,
    sum: c_double,
    sum_squares: c_double,
    min_val: c_double,
    max_val: c_double,
}

impl PeaksCD {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            sum: 0.0,
            sum_squares: 0.0,
            min_val: c_double::INFINITY,
            max_val: c_double::NEG_INFINITY,
        }
    }
    
    fn from_data(data: &[c_double]) -> Self {
        let mut peaks = Self::new();
        for &value in data {
            peaks.add(value);
        }
        peaks
    }
    
    fn add(&mut self, value: c_double) {
        self.data.push(value);
        self.sum += value;
        self.sum_squares += value * value;
        self.min_val = self.min_val.min(value);
        self.max_val = self.max_val.max(value);
    }
    
    fn mean(&self) -> c_double {
        if self.data.is_empty() {
            c_double::NAN
        } else {
            self.sum / self.data.len() as c_double
        }
    }
    
    fn min(&self) -> c_double {
        self.min_val
    }
    
    fn max(&self) -> c_double {
        self.max_val
    }
    
    fn len(&self) -> usize {
        self.data.len()
    }
}

// Grimshaw W function using c_double
fn grimshaw_w_cd(gamma: c_double, peaks: &PeaksCD) -> c_double {
    if gamma == 0.0 {
        // Exponential case: W(0) = n - sum(x_i)
        peaks.len() as c_double - peaks.sum
    } else {
        // GPD case: W(gamma) = sum(log(1 + gamma * x_i)) / gamma
        let log_sum: c_double = peaks.data.iter()
            .map(|&x| (1.0 + gamma * x).ln())
            .sum();
        peaks.len() as c_double - log_sum / gamma
    }
}

// Brent's method using c_double (exact C implementation)
fn brent_cd<F>(a: c_double, b: c_double, f: F, epsilon: c_double) -> Option<c_double>
where
    F: Fn(c_double) -> c_double,
{
    let mut a = a;
    let mut b = b;
    let mut fa = f(a);
    let mut fb = f(b);
    
    if fa * fb > 0.0 {
        return None; // No root in interval
    }
    
    if fa.abs() < fb.abs() {
        std::mem::swap(&mut a, &mut b);
        std::mem::swap(&mut fa, &mut fb);
    }
    
    let mut c = a;
    let mut fc = fa;
    let mut d = b - a;
    let mut e = d;
    
    for _ in 0..BRENT_ITMAX {
        if fb.abs() < epsilon {
            return Some(b);
        }
        
        if (b - a).abs() < epsilon {
            return Some(b);
        }
        
        if fa.abs() > fb.abs() {
            a = b;
            b = c;
            c = a;
            fa = fb;
            fb = fc;
            fc = fa;
        }
        
        let tol = 2.0 * epsilon * b.abs().max(1.0);
        let m = 0.5 * (c - b);
        
        if m.abs() <= tol || fb == 0.0 {
            return Some(b);
        }
        
        if e.abs() >= tol && fa.abs() > fb.abs() {
            let s = fb / fa;
            let mut p: c_double;
            let mut q: c_double;
            
            if a == c {
                p = 2.0 * m * s;
                q = 1.0 - s;
            } else {
                let r = fa / fc;
                let t = fb / fc;
                p = s * (2.0 * m * r * (r - t) - (b - a) * (t - 1.0));
                q = (r - 1.0) * (t - 1.0) * (s - 1.0);
            }
            
            if p > 0.0 {
                q = -q;
            } else {
                p = -p;
            }
            
            if 2.0 * p < xmin_cd(3.0 * m * q - (tol * q).abs(), (e * q).abs()) {
                e = d;
                d = p / q;
            } else {
                d = m;
                e = d;
            }
        } else {
            d = m;
            e = d;
        }
        
        a = b;
        fa = fb;
        
        if d.abs() > tol {
            b += d;
        } else {
            b += if m > 0.0 { tol } else { -tol };
        }
        
        fb = f(b);
    }
    
    Some(b)
}

// Compute log-likelihood using c_double
fn compute_log_likelihood_cd(peaks: &PeaksCD, gamma: c_double, sigma: c_double) -> c_double {
    if is_nan_cd(gamma) || is_nan_cd(sigma) || sigma <= 0.0 {
        return c_double::NEG_INFINITY;
    }
    
    let n = peaks.len() as c_double;
    
    if gamma == 0.0 {
        // Exponential distribution
        -n * (1.0 + sigma.ln()) - peaks.sum / sigma
    } else {
        // GPD
        let mut log_sum = 0.0;
        for &x in &peaks.data {
            let arg = 1.0 + gamma * x / sigma;
            if arg <= 0.0 {
                return c_double::NEG_INFINITY;
            }
            log_sum += arg.ln();
        }
        -n * sigma.ln() - (1.0 + 1.0/gamma) * log_sum
    }
}

// Grimshaw estimator using c_double throughout
fn grimshaw_estimator_cd(peaks: &PeaksCD) -> (c_double, c_double, c_double) {
    let mini = peaks.min();
    let maxi = peaks.max();
    let mean = peaks.mean();
    
    if is_nan_cd(mini) || is_nan_cd(maxi) || is_nan_cd(mean) {
        return (c_double::NAN, c_double::NAN, c_double::NAN);
    }
    
    let epsilon = xmin_cd(BRENT_DEFAULT_EPSILON_CD, 0.5 / maxi);
    
    let mut found = [true, false, false];
    let mut roots = [0.0, 0.0, 0.0];
    
    // Left root
    let a = -1.0 / maxi + epsilon;
    let b = -epsilon;
    if let Some(root) = brent_cd(a, b, |x| grimshaw_w_cd(x, peaks), BRENT_DEFAULT_EPSILON_CD) {
        roots[1] = root;
        found[1] = true;
    }
    
    // Right root  
    let a = epsilon;
    let b = 2.0 * (mean - mini) / (mini * mini);
    if let Some(root) = brent_cd(a, b, |x| grimshaw_w_cd(x, peaks), BRENT_DEFAULT_EPSILON_CD) {
        roots[2] = root;
        found[2] = true;
    }
    
    // Find best root
    let mut best_gamma = roots[0];
    let mut best_sigma = mean / (1.0 + best_gamma);
    let mut max_llhood = compute_log_likelihood_cd(peaks, best_gamma, best_sigma);
    
    for k in 1..3 {
        if found[k] {
            let gamma = roots[k];
            let sigma = mean / (1.0 + gamma);
            let llhood = compute_log_likelihood_cd(peaks, gamma, sigma);
            
            if llhood > max_llhood {
                max_llhood = llhood;
                best_gamma = gamma;
                best_sigma = sigma;
            }
        }
    }
    
    (best_gamma, best_sigma, max_llhood)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== C_DOUBLE PRECISION TEST ===");
    println!("Testing if using c_double eliminates Grimshaw estimator precision differences");
    
    // Use same seed as other tests
    unsafe { srand(42) };
    
    // Create standard f64 implementation
    let config = SpotConfig::default();
    let mut detector_f64 = Spot::new(config)?;
    
    // Generate training data
    let mut training_data_f64 = Vec::with_capacity(20000);
    
    for _ in 0..20000 {
        let val_f64 = unsafe { rand() as f64 / (i32::MAX as f64 + 1.0) };
        training_data_f64.push(val_f64);
    }
    
    // Fit f64 model
    detector_f64.fit(&training_data_f64)?;
    println!("F64 model fitted. Initial threshold T = {}", detector_f64.excess_threshold());
    
    // Process samples until divergence point
    let target_step = 100000;
    let mut step_count = 0;
    let mut f64_anomaly_count = 0;
    let mut f64_excess_count = 0;
    let mut f64_normal_count = 0;
    
    for _ in 0..target_step {
        let value_f64 = unsafe { rand() as f64 / (i32::MAX as f64 + 1.0) };
        
        let status = detector_f64.step(value_f64)?;
        step_count += 1;
        
        match status {
            SpotStatus::Anomaly => f64_anomaly_count += 1,
            SpotStatus::Excess => f64_excess_count += 1,
            SpotStatus::Normal => f64_normal_count += 1,
        }
    }
    
    println!("\n=== F64 RESULTS AT STEP {} ===", step_count);
    let z_f64 = detector_f64.anomaly_threshold();
    let t_f64 = detector_f64.excess_threshold();
    let (gamma_f64, sigma_f64) = detector_f64.get_gpd_parameters();
    
    println!("Z={:.9} T={:.9} Gamma={:.15} Sigma={:.15}", z_f64, t_f64, gamma_f64, sigma_f64);
    println!("Anomaly: {}, Excess: {}, Normal: {}", f64_anomaly_count, f64_excess_count, f64_normal_count);
    
    // Test c_double Grimshaw estimator with same excess data
    println!("\n=== C_DOUBLE GRIMSHAW TEST ===");
    
    // Get excess values from f64 implementation and convert to c_double  
    let excess_f64 = detector_f64.get_excess_values();
    let excess_cd: Vec<c_double> = excess_f64.iter().map(|&x| x as c_double).collect();
    
    println!("Testing Grimshaw with {} excess values", excess_cd.len());
    
    let peaks_cd = PeaksCD::from_data(&excess_cd);
    let (gamma_cd, sigma_cd, ll_cd) = grimshaw_estimator_cd(&peaks_cd);
    
    println!("C_DOUBLE Grimshaw results:");
    println!("Gamma: {:.15}", gamma_cd);
    println!("Sigma: {:.15}", sigma_cd);
    println!("Log-likelihood: {:.15}", ll_cd);
    
    // Compare with f64 version
    println!("\n=== COMPARISON ===");
    println!("Gamma - f64: {:.15}", gamma_f64);
    println!("Gamma - c_double: {:.15}", gamma_cd);
    println!("Gamma difference: {:.2e}", (gamma_f64 as c_double - gamma_cd).abs());
    
    println!("Sigma - f64: {:.15}", sigma_f64);
    println!("Sigma - c_double: {:.15}", sigma_cd);
    println!("Sigma difference: {:.2e}", (sigma_f64 as c_double - sigma_cd).abs());
    
    // Check bit-level equality
    let gamma_f64_bits = (gamma_f64 as c_double).to_bits();
    let gamma_cd_bits = gamma_cd.to_bits();
    let sigma_f64_bits = (sigma_f64 as c_double).to_bits();
    let sigma_cd_bits = sigma_cd.to_bits();
    
    println!("\nBit-level comparison:");
    println!("Gamma - f64 bits: {:016x}", gamma_f64_bits);
    println!("Gamma - c_double bits: {:016x}", gamma_cd_bits);
    println!("Gamma bits identical? {}", gamma_f64_bits == gamma_cd_bits);
    
    println!("Sigma - f64 bits: {:016x}", sigma_f64_bits);
    println!("Sigma - c_double bits: {:016x}", sigma_cd_bits);
    println!("Sigma bits identical? {}", sigma_f64_bits == sigma_cd_bits);
    
    if gamma_f64_bits == gamma_cd_bits && sigma_f64_bits == sigma_cd_bits {
        println!("\n✅ C_DOUBLE AND F64 PRODUCE IDENTICAL RESULTS");
        println!("The precision difference is NOT due to floating-point type differences.");
        println!("The issue must be in the fundamental algorithm implementation differences.");
    } else {
        println!("\n❌ C_DOUBLE AND F64 PRODUCE DIFFERENT RESULTS");
        println!("The floating-point type difference may be contributing to the precision issue.");
    }
    
    Ok(())
}