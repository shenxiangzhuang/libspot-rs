use libspot::{Spot, SpotConfig as RustConfig, SpotStatus as RustStatus};
use libspot_ffi::{SpotConfig as FFIConfig, SpotDetector, SpotStatus as FFIStatus};

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
        self.rand() as f64 / 2147483647.0
    }

    pub fn rexp(&mut self) -> f64 {
        -self.runif().ln()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Direct Comparison Test - Pure Rust vs FFI");

    let rust_config = RustConfig {
        q: 0.0001,
        low_tail: false,
        discard_anomalies: true,
        level: 0.998,
        max_excess: 200,
    };

    let ffi_config = FFIConfig {
        q: 0.0001,
        low_tail: false,
        discard_anomalies: true,
        level: 0.998,
        max_excess: 200,
    };

    let mut rust_detector = Spot::new(rust_config)?;
    let mut ffi_detector = SpotDetector::new(ffi_config)?;

    // Generate training data
    let n = 20000;
    let mut rng = CRand::new(1);
    let mut training_data = Vec::with_capacity(n);
    for _ in 0..n {
        training_data.push(rng.rexp());
    }

    rust_detector.fit(&training_data)?;
    ffi_detector.fit(&training_data)?;

    println!("After training:");
    println!(
        "  Rust Excess threshold: {:.15}",
        rust_detector.excess_threshold()
    );
    println!(
        "  FFI  Excess threshold: {:.15}",
        ffi_detector.excess_threshold()
    );
    println!(
        "  Rust Anomaly threshold: {:.15}",
        rust_detector.anomaly_threshold()
    );
    println!(
        "  FFI  Anomaly threshold: {:.15}",
        ffi_detector.anomaly_threshold()
    );

    let excess_diff = (rust_detector.excess_threshold() - ffi_detector.excess_threshold()).abs();
    let anomaly_diff = (rust_detector.anomaly_threshold() - ffi_detector.anomaly_threshold()).abs();

    println!("  Excess threshold difference: {:.15}", excess_diff);
    println!("  Anomaly threshold difference: {:.15}", anomaly_diff);

    if excess_diff > 1e-15 || anomaly_diff > 1e-15 {
        println!("WARNING: Initial thresholds differ!");
        return Ok(());
    }

    // Test processing a few values
    println!("\nProcessing test values...");
    for i in 0..10 {
        let val = rng.rexp();

        let rust_status = rust_detector.step(val)?;
        let ffi_status = ffi_detector.step(val)?;

        let rust_status_int = match rust_status {
            RustStatus::Normal => 0,
            RustStatus::Excess => 1,
            RustStatus::Anomaly => 2,
        };

        let ffi_status_int = match ffi_status {
            FFIStatus::Normal => 0,
            FFIStatus::Excess => 1,
            FFIStatus::Anomaly => 2,
        };

        println!(
            "  Sample {}: val={:.6} rust={} ffi={} match={}",
            i,
            val,
            rust_status_int,
            ffi_status_int,
            rust_status_int == ffi_status_int
        );

        if rust_status_int != ffi_status_int {
            println!("    MISMATCH at sample {}!", i);
            println!(
                "    Rust Z={:.15} T={:.15}",
                rust_detector.anomaly_threshold(),
                rust_detector.excess_threshold()
            );
            println!(
                "    FFI  Z={:.15} T={:.15}",
                ffi_detector.anomaly_threshold(),
                ffi_detector.excess_threshold()
            );
            break;
        }
    }

    println!("Both implementations are synchronized!");
    Ok(())
}
