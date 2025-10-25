use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpStream},
};

use crate::constants;


pub async fn receive_file(file_path: &str, connection: TcpStream) {
    let (mut reader, _writer) = connection.into_split();

    let file = File::create(file_path)
        .await
        .expect("Unable to create file.");
    let mut file_writer = BufReader::new(file);
    let mut buffer = [0; constants::BUFFER_SIZE];

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .await
            .expect("Unable to read data from stream.");

        let msg = str::from_utf8(&buffer[..bytes_read])
            .map(|s| s.to_ascii_lowercase())
            .unwrap();

        println!("Received: {msg}");

        if &buffer[..bytes_read] == b"TRANSFER_COMPLETE" {
            println!("File transfer complete.");
            break;
        }

        if bytes_read == 0 {
            println!("Connection closed by server.");
            break; // Connection closed
        }

        file_writer
            .get_mut()
            .write_all(&buffer[..bytes_read])
            .await
            .expect("Unable to write data to file.");
    }
}