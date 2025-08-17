//! Shared random number generation module
//!
//! Provides C-compatible rand()/srand() functions for reproducible results
//! across all experiments.

use libc;

/// Set the random seed using C's srand()
pub fn set_random_seed(seed: u32) {
    unsafe {
        libc::srand(seed);
    }
}

/// Generate random number using C's rand()
pub fn c_random() -> f64 {
    unsafe {
        let rand_val = libc::rand();
        // Convert to uniform [0,1) matching C implementation
        (rand_val as f64) / ((libc::RAND_MAX as f64) + 1.0)
    }
}

/// Generate exponential random variable
pub fn c_random_exp() -> f64 {
    -c_random().ln()
}