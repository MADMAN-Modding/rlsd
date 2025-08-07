use std::{
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

use base64::{Engine, engine::general_purpose};
use serde_json::Value;
use sqlx::{Pool, Sqlite};

use crate::{
    json_handler::{self, ToDevice},
    socket_handling::command_type::{CommandTraits, Commands},
    stats_handling::{database, device_info::get_device_id, stats_getter},
};

#[derive(Clone)]
pub struct Receiver {
    pub exit: bool,
    pub database: Pool<Sqlite>,
}

impl Receiver {
    pub fn new(database: Pool<Sqlite>) -> Receiver {
        Receiver {
            exit: false,
            database: database,
        }
    }

    /// Starts the socket
    pub async fn start(&mut self) -> std::io::Result<()> {
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
                    eprintln!("Failed to handle the connection: {e}");
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
                    eprintln!("Failed to convert decoded bytes to String: {}", e);
                    return;
                }
            },
            Err(e) => {
                eprintln!("Failed to decode base64 string: {}", e);
                return;
            }
        };

        // Match the command to the Commands enum
        match command.to_command() {
            Commands::INPUT => self.input(stream, json_string).await,
            Commands::RENAME => self.rename(stream, json_string).await,
            Commands::SETUP => self.setup(stream).await,
            Commands::EXIT => self.exit(),
            _ => self.error(),
        }
    }

    // Takes the json data as an input and adds it to the display data
    async fn input(&mut self, mut stream: TcpStream, json_string: String) {
        // Convert json_string to a Value
        let mut json: Value = match serde_json::from_str(json_string.as_str()) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Failed to parse JSON: {}", e);
                return;
            }
        };

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

    async fn rename(&mut self, mut stream: TcpStream, json_string: String) {
        let json: Value = match serde_json::from_str(json_string.as_str()) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("Failed to parse JSON: {}", e);
                return;
            }
        };

        let device_id = json_handler::read_json_from_buf("deviceID", &json);
        let device_name = json_handler::read_json_from_buf("deviceName", &json);

        let result = database::rename_device(&self.database, &device_id, &device_name).await;

        stream.write_all(result.as_bytes()).unwrap();
    }

    fn error(&mut self) {
        eprintln!("Command not recognized!")
    }

    async fn setup(&mut self, mut stream: TcpStream) {
        let id = get_device_id(&self.database).await;

        stream.write_all(id.as_bytes()).unwrap();
    }
}
