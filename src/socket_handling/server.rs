use std::{
    collections::HashMap, io::{Read, Write}, net::{TcpListener, TcpStream}, time::Duration
};

use base64::{Engine, engine::general_purpose};
use serde_json::Value;
use sqlx::{Pool, Sqlite};
use tokio::time::sleep;

use crate::{
    config::server::ServerConfig as ServerConfig, constants::get_server_config_path, json_handler::{self, write_server_config_all, ToDevice, ToServerConfig}, socket_handling::command_type::{CommandTraits, Commands}, stats_handling::{database, device_info::get_device_id, stats_getter}
};

#[derive(Clone)]
/// Configuration for the socket part of the server
pub struct Receiver {
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

impl Receiver {
    pub fn new(database: Pool<Sqlite>, print: bool) -> Receiver {
        let config = json_handler::read_json_as_value(&get_server_config_path()).to_sever();

        Receiver {
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

    fn exit(&mut self) {
        self.exit = true;
    }

    /// Takes the stream and determines what command should be ran
    ///
    /// Decodes the base64 data to json
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
        let json_string = match decoded_bytes {
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
        
        let json: Value = match serde_json::from_str(&json_string) {
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
            Commands::INPUT => self.input(stream, json).await,
            Commands::RENAME => self.rename(stream, json).await,
            Commands::SETUP => self.setup(stream).await,
            Commands::REMOVE => self.remove_device(stream, json).await,
            Commands::LIST => self.list(stream, json).await,
            Commands::EXIT => self.exit(),
            _ => self.error(),
        }
    }

    // Takes the json data as an input and adds it to the display data
    async fn input(&mut self, mut stream: TcpStream, mut json: Value) {
        let device_id = json["deviceID"].as_str().unwrap();

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
        json["time"] = Value::Number(stats_getter::get_unix_timestamp().into());

        let device = json.to_device();

        if device.device_id != "N/A" {
            // println!("INPUT RECEIVED:\n{}\n{}\n{}", get_divider(), device.to_string().blue().bold(), get_divider());
            
            database::input_data(&self.database, device).await.ok();

            match stream.write_all("Data inserted".as_bytes()) {
                Ok(v) => v,
                _ => {}
            };
        }

        match stream.write_all("Failed to insert data".as_bytes()) {
            Ok(v) => v,
            _ => {}
        }
    }

    async fn rename(&mut self, mut stream: TcpStream, json: Value) {
        let device_id = json_handler::read_json_from_buf("deviceID", &json);
        let device_name = json_handler::read_json_from_buf("deviceName", &json);

        let result = database::rename_device(&self.database, &device_id, &device_name).await;

        stream.write_all(result.as_bytes()).unwrap();
    }

    async fn remove_device(&mut self, mut stream: TcpStream, json: Value) {
        // Get the device id or set it to N/A
        let device_id = json.get("deviceID").unwrap().as_str().unwrap_or("N/A");
        
        // Return if the id is N/A
        if device_id == "N/A" {return;}

        // If that sha256 exists in the admin list, continue
        if self.admin_check(device_id) {
            let removed_device_id = json["removedDeviceID"].as_str().unwrap();

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
            stream.write_all("You can't do that".as_bytes()).unwrap();
        }
    }

    async fn list(&mut self, mut stream: TcpStream, json: Value) {
        if !self.admin_check(json["deviceID"].as_str().unwrap()) {
            match stream.write_all("You aren't allow to do that".as_bytes()) {
                Ok(s) => s,
                Err(e) => if self.print {println!("{e}")}
            }
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

    fn error(&mut self) {
        eprintln!("Command not recognized!")
    }

    async fn setup(&mut self, mut stream: TcpStream) {
        let id = get_device_id(&self.database).await;

        self.config.registered_device_ids.push(id.clone());

        write_server_config_all(self.config.to_json());

        stream.write_all(id.as_bytes()).unwrap();
    }

    fn admin_check(&mut self, id: &str) -> bool {
        if self.config.admin_ids.contains(&id.to_string()) {
            true
        } else {
            false
        }
    }
}
