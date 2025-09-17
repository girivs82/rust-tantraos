//! TRUE Zero-Boilerplate TantraOS Tasklet
//!
//! This should compile with ZERO annotations when using rust-tantraos
//! with the aarch64-tantraos target.

// No #![no_std] - we HAVE std (std-tantraos)
// No #![no_main] - tasklet_main is the default entry
// No extern crate - std is automatic

async fn main() {
    println!("🎉 TRUE ZERO BOILERPLATE ACHIEVED!");
    println!("This is just normal async Rust!");
    println!("No annotations, no special setup!");

    // Test async operations
    for i in 1..=3 {
        println!("Async iteration {}", i);
        std::task::yield_now().await;
    }

    println!("TantraOS: The future of async OS development!");
}