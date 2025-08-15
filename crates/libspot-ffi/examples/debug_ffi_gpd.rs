//! Debug GPD parameters for FFI version

use libspot_ffi::{SpotConfig, SpotDetector, SpotStatus};

pub struct CRand;
impl CRand {
    pub fn new(seed: u32) -> Self { unsafe { libc::srand(seed); } CRand }
    pub fn rand(&mut self) -> u32 { unsafe { libc::rand() as u32 } }
    pub fn runif(&mut self) -> f64 { self.rand() as f64 / 2147483647.0 }
    pub fn rexp(&mut self) -> f64 { -self.runif().ln() }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== DEBUG FFI GPD PARAMETERS ===");
    
    let config = SpotConfig { q: 0.0001, low_tail: false, discard_anomalies: true, level: 0.998, max_excess: 200 };
    let mut detector = SpotDetector::new(config)?;
    let mut rng = CRand::new(1);

    // Training + reach 95k state
    let mut initial_data = Vec::with_capacity(20000);
    for _ in 0..20000 { initial_data.push(rng.rexp()); }
    detector.fit(&initial_data)?;
    
    // Process to 95k samples
    for _ in 0..95000 {
        let val = rng.rexp();
        detector.step(val)?;
    }
    
    println!("At 95k samples: Z={:.15}", detector.anomaly_threshold());
    
    // Process the critical 5k samples
    for i in 0..5000 {
        let val = rng.rexp();
        let status = detector.step(val)?;
        
        if i % 1000 == 0 || i == 4999 {
            println!("Sample {}: Z={:.15} T={:.15} val={:.6} status={:?}", 
                     i, detector.anomaly_threshold(), detector.excess_threshold(), val, status);
        }
    }
    
    println!("At 100k samples: Z={:.15} (expected 8.289334)", detector.anomaly_threshold());
    
    Ok(())
}