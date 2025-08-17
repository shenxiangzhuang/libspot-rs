//! Test C FFI Grimshaw estimator with the critical data
//!
//! This program loads the critical peaks data and calls the C Grimshaw estimator
//! to compare results with the Rust implementation.

use std::fs::File;
use std::io::{BufRead, BufReader};
use std::os::raw::{c_double, c_int, c_ulong};
use std::ptr;

// C structures matching the ones in ffi.rs
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

// FFI functions
#[link(name = "spot", kind = "static")]
extern "C" {
    pub fn grimshaw_estimator(peaks: *const Peaks, gamma: *mut c_double, sigma: *mut c_double) -> c_double;
}

/// Load peaks data from CSV file
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

/// Create a C Peaks structure with the given data
fn create_c_peaks(data: &[f64]) -> Result<Peaks, Box<dyn std::error::Error>> {
    let capacity = data.len();
    let data_ptr = unsafe {
        let ptr = libc::malloc(capacity * std::mem::size_of::<c_double>()) as *mut c_double;
        if ptr.is_null() {
            return Err("Memory allocation failed".into());
        }
        
        // Copy data
        for (i, &value) in data.iter().enumerate() {
            *ptr.add(i) = value as c_double;
        }
        
        ptr
    };
    
    let container = Ubend {
        cursor: capacity as c_ulong,
        capacity: capacity as c_ulong,
        last_erased_data: f64::NAN,
        filled: 1, // true
        data: data_ptr,
    };
    
    // Compute statistics
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

/// Free the C Peaks structure memory
fn free_c_peaks(peaks: &mut Peaks) {
    unsafe {
        if !peaks.container.data.is_null() {
            libc::free(peaks.container.data as *mut libc::c_void);
            peaks.container.data = ptr::null_mut();
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing C FFI Grimshaw estimator");
    
    // Load the critical peaks data
    let data = load_peaks_data("critical_peaks_data.csv")?;
    println!("Loaded {} peaks values", data.len());
    
    let mean = data.iter().sum::<f64>() / data.len() as f64;
    let min = data.iter().fold(f64::NAN, |a, &b| if a.is_nan() || b < a { b } else { a });
    let max = data.iter().fold(f64::NAN, |a, &b| if a.is_nan() || b > a { b } else { a });
    
    println!("Peaks mean: {:.15}", mean);
    println!("Peaks min: {:.15}", min);
    println!("Peaks max: {:.15}", max);
    
    // Create C Peaks structure
    let mut peaks = create_c_peaks(&data)?;
    
    // Call C Grimshaw estimator
    let mut gamma: c_double = 0.0;
    let mut sigma: c_double = 0.0;
    
    let llhood = unsafe {
        grimshaw_estimator(&peaks as *const Peaks, &mut gamma as *mut c_double, &mut sigma as *mut c_double)
    };
    
    println!("C FFI Grimshaw Results:");
    println!("Gamma: {:.15}", gamma);
    println!("Sigma: {:.15}", sigma);
    println!("Log-likelihood: {:.15}", llhood);
    
    // Compute the anomaly threshold like SPOT does
    let q = 0.0001;
    let nt = 166_f64; // Number of Z updates
    let n = 97066_f64; // Total number of seen data
    let excess_threshold = 6.236165177550786;
    let up_down = 1.0; // For upper tail
    
    let s = nt / n;
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
    
    // Free memory
    free_c_peaks(&mut peaks);
    
    Ok(())
}