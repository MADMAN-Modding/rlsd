use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::encryption::{decrypt_small_file, derive_chunk_nonce, EncryptionKeys};

pub async fn receive_file(file_path: &str, connection: TcpStream, keys: EncryptionKeys) {
    let (mut reader, _writer) = connection.into_split();

    let file = File::create(file_path)
        .await
        .expect("Unable to create file.");
    let mut file_writer = BufReader::new(file);

    let key: [u8; 32] = keys.aes_key[0..32].try_into().unwrap();
    let nonce: [u8; 24] = keys.aes_nonce[0..24].try_into().unwrap();
    drop(keys);

    let mut chunk_index = 0u64;

    loop {
        // read length prefix
        let mut len_buf = [0u8; 4];
        if let Err(_) = reader.read_exact(&mut len_buf).await {
            println!("Connection closed by sender.");
            break;
        }
        let chunk_len = u32::from_le_bytes(len_buf) as usize;

        // read full ciphertext
        let mut enc_data = vec![0u8; chunk_len];
        if let Err(_) = reader.read_exact(&mut enc_data).await {
            eprintln!("Connection closed mid-chunk");
            break;
        }

        // derive nonce for this chunk
        let chunk_nonce = derive_chunk_nonce(&nonce, chunk_index);

        // decrypt
        let decrypted_data =
            decrypt_small_file(&enc_data, &key, &chunk_nonce).expect("Failed to decrypt chunk");

        file_writer
            .get_mut()
            .write_all(&decrypted_data)
            .await
            .expect("Unable to write data to file.");

        chunk_index += 1;
    }
}
