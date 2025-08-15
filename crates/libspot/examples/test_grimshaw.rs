use libspot::peaks::Peaks;
use libspot::estimator::grimshaw_estimator;
use std::env;

/// Random number generator that matches C's rand()/srand()
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
        self.rand() as f64 / 2147483647.0
    }

    pub fn rexp(&mut self) -> f64 {
        -self.runif().ln()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env::set_var("SPOT_DEBUG_GRIMSHAW", "1");
    
    let mut rng = CRand::new(1);
    
    // Create a peaks structure with some excess data
    let mut peaks = Peaks::new(200)?;
    
    // Add some data similar to what would be in the peaks during the run
    for _ in 0..50 {
        let val = rng.rexp();
        if val > 6.236165 { // Simulate excess data
            peaks.push(val - 6.236165); // Store as difference from threshold
        }
    }
    
    println!("Peaks data: min={:.15}, max={:.15}, mean={:.15}, size={}", 
             peaks.min(), peaks.max(), peaks.mean(), peaks.size());
    
    // Run Grimshaw estimator
    let (gamma, sigma, llhood) = grimshaw_estimator(&peaks);
    println!("Grimshaw result: gamma={:.15}, sigma={:.15}, llhood={:.15}", gamma, sigma, llhood);
    
    Ok(())
}