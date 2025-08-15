//! Zoom in on 50k-100k range to find exact divergence point

use libspot_ffi::{SpotConfig, SpotDetector, SpotStatus};

pub struct CRand;
impl CRand {
    pub fn new(seed: u32) -> Self { unsafe { libc::srand(seed); } CRand }
    pub fn rand(&mut self) -> u32 { unsafe { libc::rand() as u32 } }
    pub fn runif(&mut self) -> f64 { self.rand() as f64 / 2147483647.0 }
    pub fn rexp(&mut self) -> f64 { -self.runif().ln() }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== ZOOMING IN: FFI 50k-100k RANGE ===");
    
    let config = SpotConfig { q: 0.0001, low_tail: false, discard_anomalies: true, level: 0.998, max_excess: 200 };
    let mut detector = SpotDetector::new(config)?;
    let mut rng = CRand::new(1);

    // Training + reach 50k state
    let mut initial_data = Vec::with_capacity(20000);
    for _ in 0..20000 { initial_data.push(rng.rexp()); }
    detector.fit(&initial_data)?;
    
    // Process 50k samples to reach the "good state"
    for _ in 0..50000 {
        let val = rng.rexp();
        detector.step(val)?;
    }
    
    println!("At 50k samples: Z={:.6} T={:.6}", detector.anomaly_threshold(), detector.excess_threshold());
    
    // Now check every 5k samples in the critical 50k-100k range
    let mut total_a = 12; // Starting values from previous run
    let mut total_e = 94;
    let mut total_n = 59894;
    
    for step in 1..=10 {
        let mut anomaly = 0;
        let mut excess = 0;
        let mut normal = 0;
        
        for _ in 0..5000 {
            let val = rng.rexp();
            match detector.step(val)? {
                SpotStatus::Normal => normal += 1,
                SpotStatus::Excess => excess += 1,
                SpotStatus::Anomaly => anomaly += 1,
            }
        }
        
        total_a += anomaly;
        total_e += excess;
        total_n += normal;
        
        let current_samples = 50000 + (step * 5000);
        println!("At {:5}k: +A={:2} +E={:2} +N={:4} | Total: A={:3} E={:3} N={:6} | Z={:.6} T={:.6}",
                 current_samples / 1000, anomaly, excess, normal,
                 total_a, total_e, total_n,
                 detector.anomaly_threshold(), detector.excess_threshold());
    }
    
    Ok(())
}