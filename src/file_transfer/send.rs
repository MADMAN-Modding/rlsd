use tokio::{fs::File, io::{AsyncReadExt, AsyncWriteExt, BufReader}, net::TcpStream};

use crate::constants;

pub async fn send_file(mut stream: TcpStream, file_path: &str) {
    let file = File::open(file_path).await.expect("Unable to open file.");

    let mut reader = BufReader::new(file);
    let mut buffer = [0; constants::BUFFER_SIZE];

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .await
            .expect("Unable to read data from file.");

        if bytes_read == 0 {
            stream.flush().await.expect("Unable to flush stream.");

            break; // EOF reached
        }

        stream
            .write_all(&buffer[..bytes_read])
            .await
            .expect("Unable to write data to stream.");

        buffer.fill(0);
    }
}