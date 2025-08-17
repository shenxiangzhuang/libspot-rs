//! Test to compare logarithm implementations and quantile calculations

use std::error::Error;
use libspot::math::xlog;
use libspot_ffi::ffi;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== LOGARITHM IMPLEMENTATION COMPARISON ===");
    
    // Test the exact value from the quantile precision test
    let r = 0.020580000000000_f64;
    
    // Rust standard library
    let rust_std_ln = r.ln();
    println!("Rust std ln({:.15}) = {:.15}", r, rust_std_ln);
    
    // My custom xlog implementation  
    let my_xlog = xlog(r);
    println!("My xlog({:.15})    = {:.15}", r, my_xlog);
    
    // C library xlog through FFI
    let c_xlog = unsafe { ffi::xlog(r) };
    println!("C xlog({:.15})      = {:.15}", r, c_xlog);
    
    // Compare all three
    println!("Rust std vs my xlog: {:.2e}", my_xlog - rust_std_ln);
    println!("Rust std vs C xlog:  {:.2e}", c_xlog - rust_std_ln);
    println!("My xlog vs C xlog:   {:.2e}", c_xlog - my_xlog);
    
    // Test the critical mathematical components directly
    println!("\n=== CRITICAL CALCULATION COMPONENTS ===");
    let sigma = 0.001557685692297_f64; 
    
    let tail_quantile_rust_std = -sigma * rust_std_ln;
    let tail_quantile_my_xlog = -sigma * my_xlog;
    let tail_quantile_c_xlog = -sigma * c_xlog;
    
    println!("Tail quantile with Rust std: {:.15}", tail_quantile_rust_std);
    println!("Tail quantile with my xlog:  {:.15}", tail_quantile_my_xlog);
    println!("Tail quantile with C xlog:   {:.15}", tail_quantile_c_xlog);
    
    println!("Tail quantile differences:");
    println!("  My xlog vs Rust std: {:.2e}", tail_quantile_my_xlog - tail_quantile_rust_std);
    println!("  C xlog vs Rust std:  {:.2e}", tail_quantile_c_xlog - tail_quantile_rust_std);
    println!("  C xlog vs my xlog:   {:.2e}", tail_quantile_c_xlog - tail_quantile_my_xlog);
    
    let excess_threshold = 0.997164614148004_f64;
    let final_z_rust_std = excess_threshold + tail_quantile_rust_std;
    let final_z_my_xlog = excess_threshold + tail_quantile_my_xlog;
    let final_z_c_xlog = excess_threshold + tail_quantile_c_xlog;
    
    println!("\nFinal Z calculations:");
    println!("Final Z with Rust std:  {:.15}", final_z_rust_std);
    println!("Final Z with my xlog:   {:.15}", final_z_my_xlog);
    println!("Final Z with C xlog:    {:.15}", final_z_c_xlog);
    
    println!("Final Z differences:");
    println!("  My xlog vs Rust std: {:.2e}", final_z_my_xlog - final_z_rust_std);
    println!("  C xlog vs Rust std:  {:.2e}", final_z_c_xlog - final_z_rust_std);
    println!("  C xlog vs my xlog:   {:.2e}", final_z_c_xlog - final_z_my_xlog);
    
    // Check if the C xlog result matches the expected FFI result
    println!("\n=== EXPECTED RESULTS VERIFICATION ===");
    println!("Expected FFI result:      1.003213797524437");
    println!("Calculated with C xlog:   {:.15}", final_z_c_xlog);
    println!("Difference from expected: {:.2e}", final_z_c_xlog - 1.003213797524437);
    
    Ok(())
}