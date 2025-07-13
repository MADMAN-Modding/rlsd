use std::{thread::{self}, time::Duration};

use systemstat::{Platform, System};

/// Returns the user usage of the CPU
/// 
/// Will return 0 if it fails
pub fn get_user_cpu_usage() -> f32 {
    // Gets a new Platform Implementation
    let sys = System::new();

    // Starts measuring CPU load
    let cpu_avg = sys.cpu_load_aggregate().unwrap();
    // Measure for one second
    thread::sleep(Duration::from_secs(1));

    // Return the user CPU usage if Ok
    // Returns 0 if it is Err
    match cpu_avg.done() {
        Ok(v) => v.user,
        Err(_) => 0.0
    }
}

/// Returns the system RAM usage
/// 
/// Will return 0 if it fails 
pub fn get_ram_usage() -> u64 {
    // Gets a new Platform Implementation
    let sys = System::new();

    // Gets wrapped memory
    let ram_usage = sys.memory();

    // If ram_usage is Ok then it takes the total RAM subtracted by the free RAM to find the used RAM
    // Otherwise it returns 0
    match ram_usage {
        Ok(v) => v.total.0 - v.free.0,
        Err(_) => 0
    }
}