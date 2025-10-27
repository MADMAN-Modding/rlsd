use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpStream},
};

use crate::{constants, encryption::{decrypt_small_file, EncryptionKeys}};

pub async fn receive_file(file_path: &str, connection: TcpStream, keys: EncryptionKeys) {
    let (mut reader, _writer) = connection.into_split();

    let file = File::create(file_path)
        .await
        .expect("Unable to create file.");
    let mut file_writer = BufReader::new(file);
    let mut buffer = [0; constants::BUFFER_SIZE];

    let key: [u8; 32] = keys.aes_key[0..32].try_into().unwrap();
    let nonce: [u8; 24] = keys.aes_nonce[0..24].try_into().unwrap();

    println!("Decrypting file!");

    loop {
        let bytes_read = match reader
            .read(&mut buffer)
            .await {
                Ok(v) => v,
                Err(e) => {println!("Error trying to receive data: {:?}", e); return;}
            };

        if &buffer[..bytes_read] == b"TRANSFER_COMPLETE" {
            println!("File transfer complete.");
            break;
        }

        if bytes_read == 0 {
            println!("Connection closed by server.");
            break; // Connection closed
        }

        let enc_data = &buffer[..bytes_read].to_vec();

        let data = match decrypt_small_file(enc_data, &key, &nonce) {
            Ok(v) => v,
            Err(e) => {eprintln!("{:?}", e); return;}
        };

        file_writer
            .get_mut()
            .write_all(&data)
            .await
            .expect("Unable to write data to file.");
    }

    println!("Finished decrypting file!");

}