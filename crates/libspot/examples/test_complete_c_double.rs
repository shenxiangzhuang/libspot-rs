//! Test complete SPOT implementation using c_double throughout
//!
//! This implements a version of the SPOT algorithm that uses c_double throughout
//! to test if this eliminates the precision differences with the C implementation.

use std::os::raw::c_double;

extern "C" {
    fn srand(seed: u32);
    fn rand() -> i32;
}

fn c_rand() -> c_double {
    unsafe { rand() as c_double / (i32::MAX as c_double + 1.0) }
}

// All the c_double implementations from previous test...
// [Previous c_double implementations here - truncated for brevity]

// Simple SPOT implementation using c_double  
struct SpotCD {
    q: c_double,
    level: c_double,
    discard_anomalies: bool,
    low: bool,
    anomaly_threshold: c_double,
    excess_threshold: c_double,
    nt: usize,
    n: usize,
    excess_data: Vec<c_double>,
    max_excess: usize,
}

impl SpotCD {
    fn new() -> Self {
        Self {
            q: 0.0001,
            level: 0.998,
            discard_anomalies: true,
            low: false,
            anomaly_threshold: c_double::INFINITY,
            excess_threshold: c_double::INFINITY,
            nt: 0,
            n: 0,
            excess_data: Vec::new(),
            max_excess: 200,
        }
    }
    
    fn fit(&mut self, data: &[c_double]) -> Result<(), &'static str> {
        if data.is_empty() {
            return Err("Data cannot be empty");
        }
        
        // Simple quantile estimation - just sort and pick
        let mut sorted_data = data.to_vec();
        sorted_data.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        
        let q_index = ((1.0 - self.q) * data.len() as c_double) as usize;
        self.excess_threshold = sorted_data[q_index.min(sorted_data.len() - 1)];
        
        println!("C_DOUBLE fit completed. Excess threshold: {:.15}", self.excess_threshold);
        Ok(())
    }
    
    fn step(&mut self, value: c_double) -> Result<u8, &'static str> {
        self.n += 1;
        
        if value > self.excess_threshold {
            let excess = value - self.excess_threshold;
            
            // Add to excess buffer
            if self.excess_data.len() >= self.max_excess {
                self.excess_data.remove(0);
            }
            self.excess_data.push(excess);
            
            self.nt += 1;
            
            // Update GPD parameters
            self.update_gpd_parameters();
            
            // Check for anomaly
            if value > self.anomaly_threshold {
                if !self.discard_anomalies {
                    // Keep anomaly in excess data for threshold update
                }
                return Ok(2); // Anomaly
            } else {
                return Ok(1); // Excess
            }
        }
        
        Ok(0) // Normal
    }
    
    fn update_gpd_parameters(&mut self) {
        if self.excess_data.len() < 10 {
            return;
        }
        
        // Use the c_double Grimshaw estimator
        let peaks = PeaksCD::from_data(&self.excess_data);
        let (gamma, sigma, _) = grimshaw_estimator_cd(&peaks);
        
        if !gamma.is_finite() || !sigma.is_finite() || sigma <= 0.0 {
            return;
        }
        
        // Compute anomaly threshold using GPD quantile
        let p = self.level;
        let quantile = if gamma == 0.0 {
            // Exponential case
            sigma * (-((1.0 - p) * self.nt as c_double / self.n as c_double).ln())
        } else {
            // GPD case
            (sigma / gamma) * (((1.0 - p) * self.nt as c_double / self.n as c_double).powf(-gamma) - 1.0)
        };
        
        self.anomaly_threshold = self.excess_threshold + quantile;
    }
    
    fn get_results(&self) -> (c_double, c_double) {
        let peaks = PeaksCD::from_data(&self.excess_data);
        let (gamma, sigma, _) = grimshaw_estimator_cd(&peaks);
        (gamma, sigma)
    }
}

// Re-include the required helper structures and functions
const BRENT_DEFAULT_EPSILON_CD: c_double = 2.0e-8;
const BRENT_ITMAX: usize = 200;

fn xmin_cd(a: c_double, b: c_double) -> c_double {
    if a.is_nan() || b.is_nan() {
        return c_double::NAN;
    }
    if a < b { a } else { b }
}

fn is_nan_cd(x: c_double) -> bool {
    x.is_nan()
}

#[derive(Debug, Clone)]
struct PeaksCD {
    data: Vec<c_double>,
    sum: c_double,
    min_val: c_double,
    max_val: c_double,
}

impl PeaksCD {
    fn from_data(data: &[c_double]) -> Self {
        let mut peaks = Self {
            data: data.to_vec(),
            sum: 0.0,
            min_val: c_double::INFINITY,
            max_val: c_double::NEG_INFINITY,
        };
        
        for &value in data {
            peaks.sum += value;
            peaks.min_val = peaks.min_val.min(value);
            peaks.max_val = peaks.max_val.max(value);
        }
        
        peaks
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

fn grimshaw_w_cd(gamma: c_double, peaks: &PeaksCD) -> c_double {
    if gamma == 0.0 {
        peaks.len() as c_double - peaks.sum
    } else {
        let log_sum: c_double = peaks.data.iter()
            .map(|&x| (1.0 + gamma * x).ln())
            .sum();
        peaks.len() as c_double - log_sum / gamma
    }
}

fn brent_cd<F>(a: c_double, b: c_double, f: F, epsilon: c_double) -> Option<c_double>
where
    F: Fn(c_double) -> c_double,
{
    let mut a = a;
    let mut b = b;
    let mut fa = f(a);
    let mut fb = f(b);
    
    if fa * fb > 0.0 {
        return None;
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

fn compute_log_likelihood_cd(peaks: &PeaksCD, gamma: c_double, sigma: c_double) -> c_double {
    if is_nan_cd(gamma) || is_nan_cd(sigma) || sigma <= 0.0 {
        return c_double::NEG_INFINITY;
    }
    
    let n = peaks.len() as c_double;
    
    if gamma == 0.0 {
        -n * (1.0 + sigma.ln()) - peaks.sum / sigma
    } else {
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
    println!("=== COMPLETE C_DOUBLE SPOT TEST ===");
    println!("Testing complete SPOT implementation using c_double throughout");
    
    // Use same seed as other tests
    unsafe { srand(42) };
    
    let mut detector_cd = SpotCD::new();
    
    // Generate training data
    let mut training_data_cd = Vec::with_capacity(20000);
    for _ in 0..20000 {
        training_data_cd.push(c_rand());
    }
    
    // Fit model
    detector_cd.fit(&training_data_cd)?;
    
    // Process samples
    let target_step = 100000;
    let mut anomaly_count = 0;
    let mut excess_count = 0;
    let mut normal_count = 0;
    
    for _ in 0..target_step {
        let value = c_rand();
        let status = detector_cd.step(value)?;
        
        match status {
            2 => anomaly_count += 1,  // Anomaly
            1 => excess_count += 1,   // Excess
            0 => normal_count += 1,   // Normal
            _ => {}
        }
    }
    
    let (gamma_cd, sigma_cd) = detector_cd.get_results();
    
    println!("\n=== C_DOUBLE SPOT RESULTS AT STEP {} ===", target_step);
    println!("Z={:.9} T={:.9}", detector_cd.anomaly_threshold, detector_cd.excess_threshold);
    println!("Gamma={:.15} Sigma={:.15}", gamma_cd, sigma_cd);
    println!("Anomaly: {}, Excess: {}, Normal: {}", anomaly_count, excess_count, normal_count);
    
    println!("\n=== EXPECTED FFI RESULTS (FOR COMPARISON) ===");
    println!("Z=1.001077587 T=0.998003415");
    println!("Gamma=0.000000000000000 Sigma=0.000986623595637");
    println!("Anomaly: 3, Excess: 220, Normal: 99777");
    
    println!("\n=== ANALYSIS ===");
    println!("This test shows whether using c_double throughout eliminates");
    println!("the precision differences seen in the Grimshaw estimator.");
    
    if (gamma_cd - 0.0).abs() < 1e-15 {
        println!("✅ C_DOUBLE produces gamma=0 exactly (like FFI)");
    } else {
        println!("❌ C_DOUBLE still produces non-zero gamma: {:.2e}", gamma_cd);
    }
    
    Ok(())
}