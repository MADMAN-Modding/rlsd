use tokio::{io::{AsyncReadExt, AsyncWriteExt}, net::TcpStream};

use base64::{Engine, engine::general_purpose};
use serde_json::Value;

use crate::{constants, file_transfer::receive, json_handler::read_client_config_string, socket_handling::command_type::Commands};

/// Sends data to the socket
pub async fn send(command: Commands, data: Value) -> String {
    let server_addr = read_client_config_string("serverAddr");

    let mut connection = match connect(&server_addr).await {
        Ok(c) => c,
        Err(e) => {eprintln!("{e}"); return "Error connecting...".to_string()}
    };

    let string_json = data.to_string();

    // All of these are made to preserve temporary values
    let encoded_data = general_purpose::STANDARD.encode(string_json);

    let formatted = format!("{}{encoded_data}", command.to_string());

    let buf = formatted.as_bytes();

    // Writes the data to the stream from the buffer
    connection.write_all(buf).await.unwrap();

    let mut buf = [0; constants::BUFFER_SIZE];

    connection.read(&mut buf).await.unwrap();

    String::from_utf8_lossy(&buf)
        .trim()
        .to_string()
        .chars()
        .filter(|&c| c != '\u{0000}')
        .collect::<String>()
}

pub async fn download_database(payload: Value) -> Result<(), anyhow::Error> {
    let server_addr = read_client_config_string("serverAddr");

    let mut connection = connect(&server_addr).await.expect("Error connecting");

    let string_json = payload.to_string();

    let encoded_data = general_purpose::STANDARD.encode(string_json);

    let formatted_payload = format!("{}{encoded_data}", Commands::DownloadDatabase.to_string());

    let mut buf = formatted_payload.as_bytes();

    connection.write_all(&mut buf).await.unwrap();

    // let mut buf = [0u8; constants::BUFFER_SIZE];

    receive::receive_file("test.db", connection).await;

    Ok(())
}

pub async fn setup(server_addr: &str) -> String {
    // Used to get the device id
    let mut connection = match connect(server_addr).await {
        Ok(c) => c,
        Err(e) => {eprintln!("{e}"); return "Error".to_string();} 
    };

    connection
        .write_all(Commands::SETUP.to_string().as_bytes()).await
        .unwrap();

    let mut buf = [0; constants::BUFFER_SIZE];

    connection.read(&mut buf).await.unwrap();

    let device_id = String::from_utf8_lossy(&buf)
        .trim()
        .to_string()
        .chars()
        .filter(|&c| c != '\u{0000}')
        .collect::<String>();

    device_id
}

pub async fn connect(server_addr: &str) -> Result<TcpStream, String> {    
    match TcpStream::connect(format!("{}", server_addr)).await {
        Ok(c) => Ok(c),
        Err(e) => {
            eprintln!("{e}");
            return Err("Error".to_string());
        }
    }
}