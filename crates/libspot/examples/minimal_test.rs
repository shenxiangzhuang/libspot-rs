//! Create a minimal test case that isolates the Grimshaw estimator

use libspot::{Spot, SpotConfig};

pub struct CRand;
impl CRand {
    pub fn new(seed: u32) -> Self { unsafe { libc::srand(seed); } CRand }
    pub fn rand(&mut self) -> u32 { unsafe { libc::rand() as u32 } }
    pub fn runif(&mut self) -> f64 { self.rand() as f64 / 2147483647.0 }
    pub fn rexp(&mut self) -> f64 { -self.runif().ln() }
}

// I need to access the internal peaks data to test Grimshaw directly
// This requires adding debug methods to the library
// For now, let me create a more targeted test

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== MINIMAL GRIMSHAW TEST CASE ===");
    
    let config = SpotConfig { q: 0.0001, low_tail: false, discard_anomalies: true, level: 0.998, max_excess: 200 };
    let mut detector = Spot::new(config)?;
    let mut rng = CRand::new(1);

    // Get to exact divergence point
    let mut initial_data = Vec::with_capacity(20000);
    for _ in 0..20000 { initial_data.push(rng.rexp()); }
    detector.fit(&initial_data)?;
    
    // Process exactly 99,995 samples to get just before the critical 5 samples
    for _ in 0..99995 {
        let val = rng.rexp();
        detector.step(val)?;
    }
    
    println!("Before critical samples: Z={:.15}", detector.anomaly_threshold());
    
    // Process the exact critical samples one by one
    for i in 0..5 {
        let val = rng.rexp();
        println!("Before sample {}: val={:.15}", i, val);
        let before_z = detector.anomaly_threshold();
        
        detector.step(val)?;
        
        let after_z = detector.anomaly_threshold();
        println!("After sample {}: Z changed from {:.15} to {:.15} (diff: {:.15})", 
                 i, before_z, after_z, after_z - before_z);
    }
    
    println!("Final: Z={:.15} (Rust target: 8.287288)", detector.anomaly_threshold());
    
    Ok(())
}