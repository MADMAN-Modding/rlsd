use std::{
    io::{Read, Write},
    net::TcpStream,
};

use base64::{Engine, engine::general_purpose};
use serde_json::Value;

use crate::{json_handler::read_client_config_json, socket_handling::command_type::Commands};

/// Sends data to the socket
pub fn send(command: Commands, data: Value) {
    let addr = read_client_config_json("serverAddr");

    let mut connection = TcpStream::connect(addr).unwrap();

    let string_json = data.to_string();

    // All of these are made to preserve temporary values
    let encoded_data = general_purpose::STANDARD.encode(string_json);

    let formatted = format!("{}!{encoded_data}", command.to_string());

    let buf = formatted.as_bytes();

    // Writes the data to the stream from the buffer
    connection.write_all(buf).unwrap();
}

pub fn setup(server_addr: &str) -> String {
    // Used to get the device id
    let mut connection = match TcpStream::connect(format!("{}", server_addr)) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("{e}");
            return "Error".to_string();
        }
    };

    connection
        .write_all(format!("{}!", Commands::SETUP.to_string()).as_bytes())
        .unwrap();

    let mut buf = [0; 1024];

    connection.read(&mut buf).unwrap();

    let device_id = String::from_utf8_lossy(&buf)
        .trim()
        .to_string()
        .chars()
        .filter(|&c| c != '\u{0000}')
        .collect::<String>();

    device_id
}
