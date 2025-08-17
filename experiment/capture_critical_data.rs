//! Capture the critical peaks data where the divergence occurs
//!
//! This example runs the SPOT algorithm and captures the exact peaks data
//! at step 97066 (166th Z update) where the implementations diverge.

use libspot::{Spot, SpotConfig, SpotStatus};
use std::fs::File;
use std::io::Write;

/// Random number generator that matches C's rand()/srand() for reproducible results
pub struct CRand;

impl CRand {
    pub fn new(seed: u32) -> Self {
        unsafe {
            libc::srand(seed);
        }
        CRand
    }

    pub fn rand(&mut self) -> u32 {
        unsafe { libc::rand() as u32 }
    }

    pub fn runif(&mut self) -> f64 {
        self.rand() as f64 / 2147483647.0 // RAND_MAX = 2^31 - 1
    }

    pub fn rexp(&mut self) -> f64 {
        -self.runif().ln()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Capturing critical peaks data at step 97066...");

    // Configure SPOT detector
    let config = SpotConfig {
        q: 0.0001,               // anomaly probability
        low_tail: false,         // observe upper tail
        discard_anomalies: true, // flag anomalies
        level: 0.998,            // tail quantile
        max_excess: 200,         // data points to keep
    };

    // Create and initialize SPOT detector
    let mut detector = Spot::new(config)?;

    // Generate initial training data
    let n = 20000;
    let mut initial_data = Vec::with_capacity(n);
    let mut rng = CRand::new(1); // Use same seed as C example

    for _ in 0..n {
        initial_data.push(rng.rexp());
    }

    // Fit the model
    detector.fit(&initial_data)?;
    println!("Model fitted with {} data points", n);

    // Run until we reach step 97066 (the critical point)
    let target_step = 97066;
    let mut step_count = 0;
    let mut z_update_count = 0;

    println!("Running to step {}...", target_step);

    for i in 0..target_step {
        let val = rng.rexp();
        let old_z = detector.anomaly_threshold();
        
        match detector.step(val)? {
            SpotStatus::Normal => {
                step_count += 1;
            }
            SpotStatus::Excess => {
                step_count += 1;
                let new_z = detector.anomaly_threshold();
                if new_z != old_z {
                    z_update_count += 1;
                    if z_update_count == 166 {
                        println!("CRITICAL UPDATE {} at step {}", z_update_count, i + 1);
                        println!("Input X: {:.15}", val);
                        println!("Old Z: {:.15}", old_z);
                        println!("New Z: {:.15}", new_z);
                        
                        // Save the peaks data to CSV
                        save_peaks_data(&detector, "critical_peaks_data.csv")?;
                        println!("Critical peaks data saved to critical_peaks_data.csv");
                        
                        // Also save the exact input value
                        let mut input_file = File::create("critical_input.txt")?;
                        writeln!(input_file, "{:.15}", val)?;
                        
                        return Ok(());
                    }
                }
            }
            SpotStatus::Anomaly => {
                step_count += 1;
            }
        }
    }

    println!("Did not reach critical point - only got {} Z updates", z_update_count);
    Ok(())
}

fn save_peaks_data(detector: &Spot, filename: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Write;
    
    let mut file = File::create(filename)?;
    
    // Write CSV header
    writeln!(file, "index,value")?;
    
    // Get peaks data from the detector
    let peaks_data = detector.peaks_data();
    for (i, value) in peaks_data.iter().enumerate() {
        writeln!(file, "{},{:.15}", i, value)?;
    }
    
    // Also write metadata as comments at the end
    writeln!(file, "# Metadata")?;
    writeln!(file, "# Peaks count: {}", peaks_data.len())?;
    writeln!(file, "# Excess threshold: {:.15}", detector.excess_threshold())?;
    writeln!(file, "# Anomaly threshold: {:.15}", detector.anomaly_threshold())?;
    writeln!(file, "# Tail parameters (gamma, sigma): {:.15}, {:.15}", 
             detector.tail_parameters().0, detector.tail_parameters().1)?;
    writeln!(file, "# Peaks mean: {:.15}", detector.peaks_mean())?;
    writeln!(file, "# Peaks variance: {:.15}", detector.peaks_variance())?;
    writeln!(file, "# Peaks min: {:.15}", detector.peaks_min())?;
    writeln!(file, "# Peaks max: {:.15}", detector.peaks_max())?;
    
    Ok(())
}