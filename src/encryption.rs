use anyhow::anyhow;
use chacha20poly1305::{aead::Aead, KeyInit, XChaCha20Poly1305};
use rand::{rngs::OsRng, RngCore};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

pub struct EncryptionKeys {
    pub aes_key: Vec<u8>,
    pub aes_nonce: Vec<u8>,
}

pub fn gen_keys() -> Result<EncryptionKeys, anyhow::Error> {
    let mut key = [0u8; 32];
    let mut nonce = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);

    let aes_key = key.to_vec();
    let aes_nonce = nonce.to_vec();

    let encryption_keys = EncryptionKeys {
        aes_key,
        aes_nonce,
    };

    Ok(encryption_keys)
}

/// Uses RSA public key to encrypt the AES key and nonce
pub fn rsa_encrypt_aes_keys(
    pub_key: RsaPublicKey,
    key: &Vec<u8>,
    nonce: &Vec<u8>,
) -> Result<Vec<u8>, anyhow::Error> {
    println!("Encrypting via RSA");

    let mut rng = OsRng;

    // Encrypt each piece separately
    let key_enc = pub_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, &key)
        .expect("Failed to encrypt key");
    let nonce_enc = pub_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, &nonce)
        .expect("Failed to encrypt nonce");
    let separator = b"------";
    let separator_enc = pub_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, separator)
        .expect("Failed to encrypt separator");
    let mut buf: Vec<u8> = Vec::new();

    // Helper closure to write a lengddth-prefixed block
    let write_block = |data: &Vec<u8>, buf: &mut Vec<u8>| {
        let len = data.len() as u32;

        for byte in len.to_le_bytes() {
            buf.push(byte);
        }

        for byte in data {
            buf.push(*byte);
        }
    };

    // Write all encrypted chunks
    write_block(&key_enc, &mut buf);
    write_block(&separator_enc, &mut buf);
    write_block(&nonce_enc, &mut buf);
    
    Ok(buf)
}

pub fn rsa_decrypt(priv_key: &RsaPrivateKey, enc_data: Vec<u8>) -> Vec<Vec<u8>> {
    println!("Decrypting via RSA");

    let mut cursor = 0;
    let mut decrypted_chunks: Vec<Vec<u8>> = Vec::new();

    // Parse length-prefixed blocks until EOF
    while cursor < enc_data.iter().count() {
        if cursor + 4 > enc_data.len() {
            break;
        }
        let len = u32::from_le_bytes(enc_data[cursor..cursor + 4].try_into().unwrap()) as usize;
        cursor += 4;

        if cursor + len > enc_data.len() {
            break;
        }
        let block = &enc_data[cursor..cursor + len];
        cursor += len;

        let dec = priv_key
            .decrypt(Pkcs1v15Encrypt, block)
            .expect("Failed to decrypt block");

        println!("{}", dec.len());

        decrypted_chunks.push(dec);
    }

    println!("Decrypted {} blocks:", decrypted_chunks.len());
    decrypted_chunks
}

pub fn encrypt_small_file(
    data: Vec<u8>,
    key: &[u8; 32],
    nonce: &[u8; 24],
) -> Result<Vec<u8>, anyhow::Error> {
    let cipher = XChaCha20Poly1305::new(key.into());

    let encrypted_file = cipher
        .encrypt(nonce.into(), data.as_ref())
        .map_err(|err| anyhow!("Encrypting small file: {}", err))?;

    Ok(encrypted_file)
}

pub fn decrypt_small_file(
    enc_data: &Vec<u8>,
    key: &[u8; 32],
    nonce: &[u8; 24],
) -> Result<Vec<u8>, anyhow::Error> {
    println!("Decrypting small file");
    
    let cipher = XChaCha20Poly1305::new(key.into());

    let decrypted_file = cipher
        .decrypt(nonce.into(), enc_data.as_ref())
        .map_err(|err| anyhow!("Decrypting small file: {}", err))?;

    Ok(decrypted_file)
}