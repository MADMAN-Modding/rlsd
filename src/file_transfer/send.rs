use tokio::{fs::File, io::{AsyncReadExt, AsyncWriteExt, BufReader}, net::TcpStream};

use crate::{constants, encryption::{encrypt_small_file, EncryptionKeys}};

pub async fn send_file(mut stream: TcpStream, file_path: &str, keys: EncryptionKeys) {
    println!("Sending File!");

    let file = File::open(file_path).await.expect("Unable to open file.");

    let mut reader = BufReader::new(file);
    let mut buffer = [0; constants::BUFFER_SIZE];

    let key: [u8; 32] = keys.aes_key[0..32].try_into().unwrap();
    let nonce: [u8; 24] = keys.aes_nonce[0..24].try_into().unwrap();

    drop(keys);

    println!("Encrypting file!");

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .await
            .expect("Unable to read data from file.");

        if bytes_read == 0 {
            stream.flush().await.expect("Unable to flush stream.");

            break; // EOF reached
        }

        let data = encrypt_small_file(buffer.to_vec(), &key, &nonce).unwrap();

        stream
            .write_all(&data)
            .await
            .expect("Unable to write data to stream");

        buffer.fill(0);
    }

    println!("Encryption finished!");
}

pub async fn send_file_no_enc(mut stream: TcpStream, file_path: &str) {
    println!("Sending File!");

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