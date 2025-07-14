use std::{io::Write, net::TcpStream};

use serde_json::json;

pub fn send() {
    let mut connection = TcpStream::connect("127.0.0.1:8080").unwrap();

    let string_json = json!({"test": "key"}).to_string();
    let buf = string_json.as_bytes();

    connection.write_all(buf).unwrap();
}