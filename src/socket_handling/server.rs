use std::{
    collections::HashMap, io::{Read, Write}, net::{TcpListener, TcpStream}, time::Duration
};

use base64::{Engine, engine::general_purpose};
use serde_json::Value;
use sqlx::{Pool, Sqlite};
use tokio::time::sleep;
use whoami::Arch;

use crate::{
    config::server::ServerConfig as ServerConfig, constants::get_server_config_path, json_handler::{self, write_server_config_all, ToDevice, ToServerConfig}, socket_handling::command_type::{CommandTraits, Commands}, stats_handling::{database, device_info::get_device_id, stats_getter}
};

#[derive(Clone)]
/// Configuration for the socket part of the server
pub struct Server {
    /// `bool` - Should the server exit
    pub exit: bool,
    /// `Pool<Sqlite>` - Database to be used to execute SQL queries
    pub database: Pool<Sqlite>,
    /// `bool` - Should messages be printed
    pub print: bool,
    /// `HashMap<String, i64>` - Keeps track of when devices are sending data so it can't be spammed
    device_times: HashMap<String, i64>,
    /// `Server` - The server's config as an instance of `Server`
    config: ServerConfig
}

impl Server {
    /// Makes a new `Server` instance
    /// 
    /// # Arguments
    /// * `database: Pool<Sqlite>` - Database to execute SQL queries on
    /// * `print: bool` - Should messages be printed to the console (disable with the TUI)
    pub fn new(database: Pool<Sqlite>, print: bool) -> Server {
        let config = json_handler::read_json_as_value(&get_server_config_path()).to_server();

        Server {
            exit: false,
            database: database,
            print,
            device_times: HashMap::new(),
            config
        }
    }

    /// Starts the socket
    pub async fn start(&mut self) -> std::io::Result<()> {
        if self.config.admin_ids.is_empty() && self.print {
            println!("No admin devices found, please add at least one to allow for server management");
            sleep(Duration::from_secs(1)).await;
        }

        if self.config.registered_device_ids.is_empty() {
            let ids = database::get_all_device_uids(&self.database).await;

            for id in ids {
                self.config.registered_device_ids.push(id);
            }

            self.config.first_run = false;

            write_server_config_all(self.config.to_json());
        }

        match TcpListener::bind("0.0.0.0:51347") {
            Ok(listener) => self.handle_connection(listener).await,
            Err(e) => return Err(e),
        };

        Ok(())
    }

    /// For every stream, match it;
    /// * If it's Ok, process it
    /// * If it's Err, print the error
    /// 
    /// # Arguments
    /// * `listener: TcpListener` - Listener for incoming connections
    async fn handle_connection(&mut self, listener: TcpListener) {
        for stream in listener.incoming() {
            // If the incoming traffic is valid then process it, otherwise print an error and continue to the next loop
            match stream {
                Ok(stream) => {
                    self.process_request(stream).await;
                }
                Err(e) => {
                    if self.print {
                        eprintln!("Failed to handle the connection: {e}");
                    }
                    continue;
                }
            }

            if self.exit {
                break;
            }
        }
    }

    /// Sets the exit variable to true
    fn exit(&mut self) {
        self.exit = true;
    }

    /// Takes the stream and determines what command should be ran
    ///
    /// Decodes the base64 data to json
    /// 
    /// # Arguments
    /// * `mut stream: TcpStream` - Stream the client is connected to
    async fn process_request(&mut self, mut stream: TcpStream) {
        // Makes the buffer to store the data
        let mut buf = [0; 1024];

        // Read the data from teh stream
        stream.read(&mut buf).unwrap();

        // Raw string data
        let raw_string = String::from_utf8_lossy(&buf).trim().to_string();

        // Finds where the ! is in the msg
        let command_ending = raw_string.find("!").unwrap();

        // Gets the command the data that was encoded in base64
        let (command, encoded_data) = raw_string.split_at(command_ending + 1);

        // Decode the data from base64, the filtering removes empty bytes in the array
        let decoded_bytes = general_purpose::STANDARD.decode(
            encoded_data
                .to_string()
                .chars()
                .filter(|&c| c != '\u{0000}')
                .collect::<String>(),
        );

        // Convert the decode bytes to a string
        let payload_string = match decoded_bytes {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(s) => s,
                Err(e) => {
                    if self.print {
                        eprintln!("Failed to convert decoded bytes to String: {}", e);
                    }
                    return;
                }
            },
            Err(e) => {
                if self.print {
                    eprintln!("Failed to decode base64 string: {}", e);
                }
                return;
            }
        };
        
        let payload: Value = match serde_json::from_str(&payload_string) {
            Ok(v) => v,
            Err(e) => {
                if self.print && command.to_command() != Commands::SETUP {
                    eprintln!("Failed to parse JSON: {}", e);
                }
                Value::String("N/A".to_string())
            }
        };

        // Match the command to the Commands enum
        match command.to_command() {
            Commands::INPUT         => self.input(stream, payload).await,
            Commands::RENAME        => self.rename(stream, payload).await,
            Commands::AdminRename   => self.admin_rename(stream, payload).await,
            Commands::SETUP 		=> self.setup(stream).await,
            Commands::REMOVE 		=> self.remove_device(stream, payload).await,
            Commands::LIST 			=> self.list(stream, payload).await,
            Commands::UpdateServer  => self.update_server(stream, payload).await,
            Commands::EXIT 			=> self.exit(),
            _ => self.error(),
        }
    }

    /// Takes the json data as an input and adds it to the display data
    async fn input(&mut self, mut stream: TcpStream, mut payload: Value) {
        let device_id = payload["deviceID"].as_str().unwrap();

        // If it has been less than 110 seconds since the last time data was inserted, 
        if stats_getter::get_unix_timestamp() - self.device_times.get(device_id).unwrap_or(&0) < 110 {
            if self.print {
                println!("{device_id} tried to send data too soon");
            }
            return;
        } else {
            self.device_times.insert(device_id.to_owned(), stats_getter::get_unix_timestamp());
        }

        if !self.config.registered_device_ids.contains(&device_id.to_string()) {
            if self.print {
                println!("{device_id} tried to input data but is not registered");
            }
            return;
        }

        // Replaces the time with the server time
        payload["time"] = Value::Number(stats_getter::get_unix_timestamp().into());

        let device = payload.to_device();

        if device.device_id != "N/A" {
            // println!("INPUT RECEIVED:\n{}\n{}\n{}", get_divider(), device.to_string().blue().bold(), get_divider());
            
            database::input_data(&self.database, device).await.ok();

            match stream.write_all("Data inserted".as_bytes()) {
                Ok(v) => v,
                _ => {}
            };
        }

        self.msg_client(stream, "Failed to insert data");
    }

    /// Renames the supplied device id on the DB
    /// Sends the total amount of effected rows back to the client
    /// 
    /// # Arguments
    /// * `mut steam: TcpStream` - Stream the client is connected to
    /// * `payload: Value` - Payload from the client
    async fn rename(&mut self, mut stream: TcpStream, payload: Value) {
        let device_id = json_handler::read_json_from_buf("deviceID", &payload);
        let device_name = json_handler::read_json_from_buf("deviceName", &payload);

        let result = database::rename_device(&self.database, &device_id, &device_name).await;

        stream.write_all(result.as_bytes()).unwrap();
    }

    async fn admin_rename(&mut self, stream: TcpStream, mut payload: Value) {
        let device_id = json_handler::read_json_from_buf("deviceID", &payload);

        if self.admin_check(&device_id) {
            payload["deviceID"] = payload["renamedDeviceID"].clone();

            self.rename(stream, payload).await;
        } else {
            self.msg_client(stream, "You're not allowed to do that");
        }
    }

    /// Removes the supplied device from the registered devices and db
    /// Sends the total amount of effected rows back to the client
    /// 
    /// # Arguments
    /// * `mut steam: TcpSteam` - Stream the client is connected to
    /// * `payload: Value` - Payload from the client
    async fn remove_device(&mut self, mut stream: TcpStream, payload: Value) {
        // Get the device id or set it to N/A
        let device_id = payload.get("deviceID").unwrap().as_str().unwrap_or("N/A");
        
        // Return if the id is N/A
        if device_id == "N/A" {return;}

        // If that sha256 exists in the admin list, continue
        if self.admin_check(device_id) {
            let removed_device_id = payload["removedDeviceID"].as_str().unwrap();

            let msg = database::remove_device(&self.database, removed_device_id).await;

            // Vector to store the new ids
            let mut new_registered_device_ids: Vec<String> = Vec::new();

            // Populates the vector with all ids except the removed one
            for id in self.config.registered_device_ids.iter() {
                if id != removed_device_id {
                    new_registered_device_ids.push(id.to_owned());
                }
            }

            // Apply the new config
            self.config.registered_device_ids = new_registered_device_ids;

            // Write the new config to the config file
            write_server_config_all(self.config.to_json());

            // Try to send a message back to the client
            match stream.write_all(msg.as_bytes()) {
                Ok(s) => s,
                Err(e) => if self.print {eprintln!("{e}")}
            }
        } else {
            if self.print {
                println!("{device_id} tried to remove device data without permission")
            }
            self.msg_client(stream, "You're not allowed to do that.");
        }
    }

    /// Lists all non-admin devices on the server and sends them back over the TcpSteam
    /// 
    /// # Arguments
    /// * `mut stream: TcpSteam` - Stream the client is connected to
    /// * `payload: Value` - Payload sent by the client
    async fn list(&mut self, mut stream: TcpStream, payload: Value) {
        if !self.admin_check(payload["deviceID"].as_str().unwrap()) {
            self.msg_client(stream, "You're not allowed to do that.");
            return;
        }

        let ids = database::get_all_device_uids(&self.database).await;

        let mut msg = String::new();

        // Adds every device to the message except for admins
        for id in ids {
            if !self.admin_check(&sha256::digest(&id)) {
                msg = format!("{msg}\n{}: {}", database::get_device_name_from_uid(&self.database, &id).await, id)
            }
        }

        match stream.write_all(msg.as_bytes()) {
            Ok(s) => s,
            Err(e) => if self.print {eprintln!("{e}")}
        }
    }

    /// If the command sent isn't recognized, print a message
    fn error(&mut self) {
        if self.print {
            eprintln!("Command not recognized!")
        }
    }

    /// Updates the rlsd version on the server
    /// 
    /// # Arguments
    /// * `mut stream: TcpStream` - Stream the client is connected to
    /// * `payload: Value` - Payload sent by the client
    async fn update_server(&mut self, stream: TcpStream, payload: Value) {
        if !self.admin_check(&json_handler::read_json_from_buf("deviceID", &payload)) {
            self.msg_client(stream, "You're not allowed to do that.");
            return;
        }

        let arch = whoami::arch();
        let platform = whoami::platform();

        let binary_location = std::env::current_exe().unwrap();
        let binary_location = binary_location.to_str().unwrap(); 

        let result = match arch {
            Arch::X64 => {
                if platform == whoami::Platform::Linux {
                    download("linux/rlsd-musl", binary_location)
                } else if platform == whoami::Platform::Windows {
                    download("windows/rlsd-x86-64", binary_location)
                } else {
                    Ok(())
                }
            },
            Arch::Arm64 => {
                if platform == whoami::Platform::Linux {
                    download("linux/rlsd-aarch64", binary_location)
                } else {
                    Ok(())
                }
            }
            
            _ => Ok(())
        };

        match result {
            Ok(_) => self.msg_client(stream, "Update success"),
            Err(e) => {if self.print {
                println!("{e}")
            }}
        };
    }

    /// Makes a new id for the requesting device
    /// 
    /// # Arguments
    /// * `mut stream: TcpStream` - Stream the client is connected to
    async fn setup(&mut self, stream: TcpStream) {
        let id = get_device_id().await;

        self.config.registered_device_ids.push(id.clone());

        write_server_config_all(self.config.to_json());

        self.msg_client(stream, &id);
    }

    /// Checks to see if the supplied sha256 id is an admin
    /// 
    /// # Arguments
    /// * `id: &str` - Sha256 of the device's ID
    /// 
    /// # Returns
    /// `bool` - True if the device is an admin 
    fn admin_check(&mut self, id: &str) -> bool {
        if self.config.admin_ids.contains(&id.to_string()) {
            true
        } else {
            false
        }
    }

    fn msg_client(&mut self, mut stream: TcpStream, msg: &str) {
        match stream.write_all(msg.as_bytes()) {
            Ok(s) => s,
            Err(e) => if self.print {println!("{e}")}       
        }
    }
}

fn download(_version: &str, _file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // let url = "http://raw.githubusercontent.com/MADMAN-Modding/rlsd/refs/heads/master/bin/";

    // let response = blocking::get(&format!("{url}{version}"))?;
    // let mut dest = File::create(file_path)?;
    // let content = response.bytes()?;
    // std::io::copy(&mut content.as_ref(), &mut dest)?;
    
    Ok(())
}