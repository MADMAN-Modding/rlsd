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
    pub fn start(&mut self) -> std::io::Result<()> {
        match TcpListener::bind("127.0.0.1:8080") {
            Ok(listener) => self.handle_connection(listener),
            Err(e) => return Err(e),
        };

        Ok(())
    }

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

    fn process_request(&mut self, mut stream: TcpStream) {
        let mut buf = [0; 1024];

        stream.read(&mut buf).unwrap();

        let raw_string = String::from_utf8_lossy(&buf).trim().to_string();

        let command_ending = raw_string.find("!").unwrap();
        let (command, encoded_data) = raw_string.split_at(command_ending + 1);

        // Match the command to the Commands enum
        match command.to_command() {
            Commands::INPUT => self.input(),
            Commands::OUTPUT => self.output(encoded_data),
            Commands::EXIT => self.exit(),
            Commands::ERROR => self.error(),
        }
    }

    fn input(&mut self) {}

    fn output(&mut self, encoded_data: &str) {
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
        eprintln!("Option not recognized!")
    }
}
