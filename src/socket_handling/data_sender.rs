use std::{io::Write, net::TcpStream};

use base64::{Engine, engine::general_purpose};
use serde_json::json;

/// Sends data to the socket
pub fn send(input: &String) {
    let mut connection = TcpStream::connect("127.0.0.1:8080").unwrap();

    let string_json = json!({"test": "key"}).to_string();

    // All of these are made to preserve temporary values
    let encoded_data = general_purpose::STANDARD.encode(string_json);
    let formatted = format!("{input}!{encoded_data}");
    let buf = formatted.as_bytes();

    // Writes the data to the stream from the buffer
    connection.write_all(buf).unwrap();
}