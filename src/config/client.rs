use serde_json::{Value, json};

use crate::json_handler::ToClient;

/// Settings for a device when it's in client mode
pub struct Client {
    /// Device ID given by the server
    pub device_id: String,
    /// Friendly name for the device
    pub device_name: String,
    /// Address of the server to connect to
    pub server_addr: String,
}

impl Client {
    /// Make a new `Client` instance
    ///
    /// # Arguments
    /// * `device_id: String` - ID of the device given by the server
    /// * `device_name`: String - Friendly name fot the device
    /// * `server_addr: String` - Address of the server to connect to
    ///
    /// # Returns
    /// * A `Client` instance created from the arguments
    pub fn new(device_id: String, device_name: String, server_addr: String) -> Client {
        Client {
            device_id: device_id,
            device_name: device_name,
            server_addr: server_addr,
        }
    }

    /// Convert a `Client` instance to a `Value` instance
    pub fn to_json(&self) -> Value {
        json!({
            "deviceID"  : self.device_id,
            "deviceName": self.device_name,
            "serverAddr": self.server_addr
        })
    }

    /// Returns formatted string from the `Client` instance
    pub fn to_string(&self) -> String {
        format!(
            "Device ID: {}\nDevice Name: {}\nServer Address: {}",
            self.device_id, self.device_name, self.server_addr
        )
    }
}