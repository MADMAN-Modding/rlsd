use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::{
    constants,
    encryption::{EncryptionKeys, derive_chunk_nonce, encrypt_small_file},
};

pub async fn send_file(mut stream: TcpStream, file_path: &str, keys: EncryptionKeys) {
    let file = File::open(file_path).await.expect("Unable to open file.");
    let mut reader = BufReader::new(file);
    let mut buffer = [0u8; constants::BUFFER_SIZE];

    let key: [u8; 32] = keys.aes_key[0..32].try_into().unwrap();
    let nonce: [u8; 24] = keys.aes_nonce[0..24].try_into().unwrap();
    drop(keys);

    let mut chunk_index = 0u64;

    loop {
        let bytes_read = reader.read(&mut buffer).await.unwrap();
        if bytes_read == 0 {
            stream.flush().await.unwrap();
            break;
        }

        let chunk_nonce = derive_chunk_nonce(&nonce, chunk_index);
        let encrypted_chunk =
            encrypt_small_file(buffer[..bytes_read].to_vec(), &key, &chunk_nonce).unwrap();

        // --- send length prefix (u32 little endian) ---
        let len_bytes = (encrypted_chunk.len() as u32).to_le_bytes();
        stream.write_all(&len_bytes).await.unwrap();

        // --- send ciphertext ---
        stream.write_all(&encrypted_chunk).await.unwrap();

        chunk_index += 1;
    }
}