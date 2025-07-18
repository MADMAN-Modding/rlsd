use serde_json::{json, Value};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::{json_handler::ToDevice, stats_handling::database::check_device_id_exists};

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

    pub fn to_json(self) -> Value {
        json!({
            "deviceID": self.device_id,
            "deviceName": self.device_name,
            "ramUsed": self.ram_used,
            "ramTotal": self.ram_total,
            "cpuUsage": self.cpu_usage,
            "processes": self.processes,
            "networkIn": self.network_in,
            "networkOut": self.network_out,
            "time": self.time
        })
    }
}

impl ToDevice for serde_json::Value {
    /// Converts a JSON value to a `Device` instance.
    /// 
    /// # Returns
    /// A `Device` instance created from the JSON `Value`.
    fn to_device(&self) -> Device {
        Device {
            device_id: self["deviceID"].as_str().unwrap_or_default().to_string(),
            device_name: self["deviceName"].as_str().unwrap_or_default().to_string(),
            ram_used: self["ramUsed"].as_i64().unwrap_or(0),
            ram_total: self["ramTotal"].as_i64().unwrap_or(0),
            cpu_usage: self["cpuUsage"].as_f64().unwrap_or(0.0) as f32,
            processes: self["processes"].as_i64().unwrap_or(0) as i32,
            network_in: self["networkIn"].as_i64().unwrap_or(0),
            network_out: self["networkOut"].as_i64().unwrap_or(0),
            time: self["time"].as_i64().unwrap_or(0),
        }
    }
}

/// Returns a uuid for a device
pub async fn get_device_id(database: &Pool<Sqlite>) -> String {
    loop {
        let id = Uuid::new_v4().to_string();

        if check_device_id_exists(&id, database).await {
            return id;
        }
    }
}