//! Debug script to track Z value updates and find the divergence point
//! 
//! This script will run both implementations and track when Z values are updated,
//! specifically looking for the 201st update mentioned by the user.

use std::fs::File;
use std::io::Write;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate the same sequence of random numbers
    let mut file = File::create("/tmp/debug_z_tracking.txt")?;
    
    // Use the same random number generation as the examples
    let mut rng_rust = CRandPure::new(1);
    let mut rng_ffi = CRandFFI::new(1);
    
    // Test first few values to ensure they match
    for i in 0..10 {
        let val1 = rng_rust.rexp();
        let val2 = rng_ffi.rexp();
        writeln!(file, "Random {}: Pure={:.15} FFI={:.15} Diff={:.15}", 
                i, val1, val2, (val1 - val2).abs())?;
    }
    
    writeln!(file, "\n=== Starting Z tracking ===")?;
    
    // Now track Z updates during the actual run
    track_z_updates(&mut file)?;
    
    println!("Debug output written to /tmp/debug_z_tracking.txt");
    Ok(())
}

fn track_z_updates(file: &mut File) -> Result<(), Box<dyn std::error::Error>> {
    // This will require modifications to the actual detector implementations
    // to expose Z update tracking
    writeln!(file, "TODO: Implement Z tracking in detector implementations")?;
    Ok(())
}

// Copy the random number generators from the examples
struct CRandPure;
impl CRandPure {
    fn new(seed: u32) -> Self {
        unsafe { libc::srand(seed); }
        CRandPure
    }
    
    fn rand(&mut self) -> u32 {
        unsafe { libc::rand() as u32 }
    }
    
    fn runif(&mut self) -> f64 {
        self.rand() as f64 / 2147483647.0
    }
    
    fn rexp(&mut self) -> f64 {
        -self.runif().ln()
    }
}

struct CRandFFI;
impl CRandFFI {
    fn new(seed: u32) -> Self {
        unsafe { libc::srand(seed); }
        CRandFFI  
    }
    
    fn rand(&mut self) -> u32 {
        unsafe { libc::rand() as u32 }
    }
    
    fn runif(&mut self) -> f64 {
        self.rand() as f64 / 2147483647.0
    }
    
    fn rexp(&mut self) -> f64 {
        -self.runif().ln()
    }
}