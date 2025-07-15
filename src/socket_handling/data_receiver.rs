use std::{
    io::Read,
    net::{TcpListener, TcpStream},
};

use base64::{Engine, engine::general_purpose};
use serde_json::Value;

use crate::{
    json_handler::read_json_from_buf,
    socket_handling::command_type::{CommandTraits, Commands},
};

pub struct Receiver {
    pub exit: bool,
}

impl Receiver {
    /// Starts the socket
    pub fn start(&mut self) -> std::io::Result<()> {
        match TcpListener::bind("0.0.0.0:51347") {
            Ok(listener) => self.handle_connection(listener),
            Err(e) => return Err(e),
        };

        Ok(())
    }

    /// For every stream, match it;
    /// * If it's Ok, process it
    /// * If it's Err, print the error
    fn handle_connection(&mut self, listener: TcpListener) {
        for stream in listener.incoming() {
            // If the incoming traffic is valid then process it, otherwise print an error and continue to the next loop
            match stream {
                Ok(stream) => {
                    self.process_request(stream);
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
    fn process_request(&mut self, mut stream: TcpStream) {
        let mut buf = [0; 1024];

        stream.read(&mut buf).unwrap();

        let raw_string = String::from_utf8_lossy(&buf).trim().to_string();

        let command_ending = raw_string.find("!").unwrap();
        let (command, encoded_data) = raw_string.split_at(command_ending + 1);

        let decoded_bytes = general_purpose::STANDARD.decode(
            encoded_data
                .to_string()
                .chars()
                .filter(|&c| c != '\u{0000}')
                .collect::<String>(),
        );
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

        println!("{}", json_string);

        // Match the command to the Commands enum
        match command.to_command() {
            Commands::INPUT => self.input(),
            Commands::OUTPUT => self.output(json_string),
            Commands::EXIT => self.exit(),
            Commands::ERROR => self.error(),
        }
    }

    // Takes the json data as an input and adds it to the display data
    fn input(&mut self) {

    }

    fn output(&mut self, json_string: String) {
        let json: Value = match serde_json::from_str(json_string.as_str()) {
            Ok(val) => val,
            Err(e) => {
                eprintln!("Failed to parse JSON: {}", e);
                return;
            }
        };

        println!("{}", read_json_from_buf("test", json));
    }

    fn error(&mut self) {
        eprintln!("Command not recognized!")
    }
}
