use std::{
    thread::{self},
    time::{self, Duration, SystemTime},
};

use systemstat::Platform;

/// Returns the usage of the CPU
///
/// Will return 0 if it fails
pub fn get_cpu_usage(sys: &impl Platform) -> f32 {
    // Starts measuring CPU load
    let cpu_avg = sys.cpu_load_aggregate().unwrap();
    // Measure for one second
    thread::sleep(Duration::from_secs(1));

    // Return the CPU usage if Ok
    // Returns 0 if it is Err
    match cpu_avg.done() {
        Ok(v) => 1.0 - v.idle,
        Err(_) => 0.0,
    }
}

/// Returns the system RAM usage
///
/// Will return 0 if it fails
pub fn get_ram_usage(sys: &impl Platform) -> i64 {
    // Gets wrapped memory
    let ram_usage = sys.memory();

    // If ram_usage is Ok then it takes the total RAM subtracted by the free RAM to find the used RAM
    // Otherwise it returns 0
    match ram_usage {
        Ok(v) => (v.total.0 - v.free.0) as i64,
        Err(_) => 0,
    }
}

/// Returns the available RAM in bytes
///
/// will return 0 if it fails
pub fn get_ram_total(sys: &impl Platform) -> i64 {
    match sys.memory() {
        Ok(v) => v.total.0 as i64,
        Err(_) => 0,
    }
}

/// Returns the number of processes running on the system
pub fn get_processes() -> i32 {
    let sys = sysinfo::System::new_all();

    sys.processes().len().try_into().unwrap()
}

pub fn get_network_in(sys: &impl Platform) -> i64 {
    // Amount of bytes received
    let mut bytes_in: i64 = 0;

    for i in 0..2 {
        // For every interface, counter-pattern bytes_in by the rx bytes
        if let Ok(int) = sys.networks() {
            let interface = int.iter().next().unwrap().0.to_string();

            bytes_in += match sys.network_stats(&interface) {
                // If it's the first loop, add the bytes, if it's the second, subtract them
                Ok(v) => v.tx_bytes.0 as i64 * (if i == 0 { -1 } else { 1 }),
                Err(_) => 0,
            }
        }

        thread::sleep(Duration::from_secs(1));
    }

    bytes_in
}

pub fn get_network_out(sys: &impl Platform) -> i64 {
    // Amount of bytes transmitted
    let mut bytes_out: i64 = 0;

    for i in 0..2 {
        // For every interface, counter-pattern bytes_in by the tx bytes
        if let Ok(int) = sys.networks() {
            let interface = int.iter().next().unwrap().0.to_string();

            bytes_out += match sys.network_stats(&interface) {
                // If it's the first loop, add the bytes, if it's the second, subtract them
                Ok(v) => v.tx_bytes.0 as i64 * (if i == 0 { -1 } else { 1 }),
                Err(_) => 0,
            }
        }

        thread::sleep(Duration::from_secs(1));
    }

    bytes_out
}

/// Returns the amount of seconds since the UNIX EPOCH
pub fn get_unix_timestamp() -> i64 {
    time::SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
