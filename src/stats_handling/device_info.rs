use sqlx::{Pool, Sqlite};

/// Holds all the information about a device each minute it is monitored
#[derive(sqlx::FromRow, Clone)]
pub struct Device {
    /// Unique identifier for the device
    pub device_id: String,
    /// Friendly name for the device
    pub device_name: String,
    /// Amount of RAM used (in bytes)
    pub ram_used: i64,
    /// Amount of RAM available (in bytes)
    pub ram_total: i64,
    /// Current CPU usage as a percentage (0.0 to 1.0)
    pub cpu_usage: f32,
    /// Number of processes running on the device (this may never be used and left as NULL in the database)
    pub processes: i32,
    /// Amount of incoming network traffic (in bytes)
    pub network_in: i64,
    /// Amount of outgoing network traffic (in bytes)
    pub network_out: i64,
    /// Unix timestamp the data was taken
    pub time: i64
}

impl Device {
    /// Creates a new `Device` instance with the provided parameters.
    ///
    /// # Arguments
    ///
    /// * `device_id` - Unique identifier for the device
    /// * `device_name` - Friendly name for the device
    /// * `ram_used` - Amount of RAM currently used (in bytes)
    /// * `ram_total` - Total amount of RAM available (in bytes)
    /// * `cpu_usage` - Current CPU usage as a percentage (0.0 to 1.0)
    /// * `processes` - Number of running processes on the device
    /// * `network_in` - Amount of incoming network traffic (in bytes)
    /// * `network_out` - Amount of outgoing network traffic (in bytes)
    /// * `time` - Unix timestamp the data was taken
    ///
    /// # Returns
    ///
    /// A new `Device` instance initialized with the specified values.
    pub fn new(device_id: String, device_name: String, ram_used: i64, ram_total: i64, cpu_usage: f32, processes: i32, network_in: i64, network_out: i64, time: i64) -> Device {
        Device { device_id: device_id, device_name: device_name, ram_used: ram_used, ram_total: ram_total, cpu_usage: cpu_usage, processes: processes, network_in: network_in, network_out: network_out, time: time }
    }
}

pub async fn input_data(device: Device, database: Pool<Sqlite>) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO devices (device_id, device_name, ram_used, ram_total, cpu_usage, processes, network_in, network_out, time)
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
        "#
    )
    .bind(device.device_id)
    .bind(device.device_name)
    .bind(device.ram_used)
    .bind(device.ram_total)
    .bind(device.cpu_usage)
    .bind(device.processes)
    .bind(device.network_in)
    .bind(device.network_out)
    .bind(device.time).execute(&database)
    .await?;

    Ok(())
}