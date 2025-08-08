use serde_json::{json, Value};

#[derive(Clone)]
pub struct Server {
    /// Devices registered to connect 
    pub registered_device_ids: Vec<String>,

    /// Device IDs that have admin on the server such as remote access the rlsd settings
    /// should NOT be every device
    pub admin_ids: Vec<String>,

    /// If this is the first run of the server, it will check if a DB exists, if it does, it will add all device IDs to the list of trusted devices
    pub first_run: bool
}

impl Server {
    /// Make a new `Server` instance
    /// 
    /// # Arguments
    /// * `registered_device_ids: Vec<String>` - List of device IDs registered
    /// * `admin_ids: Vec<String>` - List of device IDs that have admin access
    /// * `first_run: bool` - If this is the first run of the server
    /// 
    /// # Returns
    /// * A `Server` instance created from the arguments
    pub fn new(registered_device_ids: Vec<String>, admin_ids: Vec<String>, first_run: bool) -> Server {
        Server {
            registered_device_ids,
            admin_ids,
            first_run,
        }
    }

    /// Convert a `Server` instance to a `Value` instance
    pub fn to_json(&self) -> Value {
        json!({
            "registeredDeviceIDs": self.registered_device_ids,
            "adminID": self.admin_ids,
            "firstRun": self.first_run
        })
    }
}