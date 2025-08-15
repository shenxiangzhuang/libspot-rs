//! Top-down debugging: Pure Rust implementation

use libspot::{Spot, SpotConfig, SpotStatus};

pub struct CRand;

impl CRand {
    pub fn new(seed: u32) -> Self {
        unsafe { libc::srand(seed); }
        CRand
    }

    pub fn rand(&mut self) -> u32 {
        unsafe { libc::rand() as u32 }
    }

    pub fn runif(&mut self) -> f64 {
        self.rand() as f64 / 2147483647.0
    }

    pub fn rexp(&mut self) -> f64 {
        -self.runif().ln()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== PURE RUST IMPLEMENTATION ===");
    
    let config = SpotConfig {
        q: 0.0001, low_tail: false, discard_anomalies: true,
        level: 0.998, max_excess: 200,
    };

    let mut detector = Spot::new(config)?;
    let mut rng = CRand::new(1);

    // Training phase
    let mut initial_data = Vec::with_capacity(20000);
    for _ in 0..20000 {
        initial_data.push(rng.rexp());
    }
    detector.fit(&initial_data)?;
    println!("After training: T = {:.6}", detector.excess_threshold());
    
    // Progressive testing to find divergence point
    let test_points = [10000, 50000, 100000, 500000, 1000000, 5000000, 10000000];
    let mut cumulative_normal = 0;
    let mut cumulative_excess = 0;
    let mut cumulative_anomaly = 0;
    
    for &n in &test_points {
        let mut normal = 0;
        let mut excess = 0;
        let mut anomaly = 0;
        
        for _ in 0..n {
            let val = rng.rexp();
            match detector.step(val)? {
                SpotStatus::Normal => normal += 1,
                SpotStatus::Excess => excess += 1,
                SpotStatus::Anomaly => anomaly += 1,
            }
        }
        
        cumulative_normal += normal;
        cumulative_excess += excess;
        cumulative_anomaly += anomaly;
        
        println!("Step {:8}: +A={:4} +E={:4} +N={:7} | Total: A={:5} E={:5} N={:8} | Z={:.6} T={:.6}", 
                 n, anomaly, excess, normal,
                 cumulative_anomaly, cumulative_excess, cumulative_normal,
                 detector.anomaly_threshold(), detector.excess_threshold());
    }
    
    // Continue to reach 50M total
    let remaining = 50_000_000 - cumulative_normal - cumulative_excess - cumulative_anomaly;
    if remaining > 0 {
        let mut normal = 0;
        let mut excess = 0;
        let mut anomaly = 0;
        
        for _ in 0..remaining {
            let val = rng.rexp();
            match detector.step(val)? {
                SpotStatus::Normal => normal += 1,
                SpotStatus::Excess => excess += 1,
                SpotStatus::Anomaly => anomaly += 1,
            }
        }
        
        cumulative_normal += normal;
        cumulative_excess += excess;
        cumulative_anomaly += anomaly;
    }
    
    println!("\nFINAL RUST: ANOMALY={} EXCESS={} NORMAL={} Z={:.6} T={:.6}",
             cumulative_anomaly, cumulative_excess, cumulative_normal,
             detector.anomaly_threshold(), detector.excess_threshold());
    
    Ok(())
}