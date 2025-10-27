use std::{
    fs::{self, File},
    io::Read,
};

use anyhow::{anyhow};
use chacha20poly1305::{aead::Aead, KeyInit, XChaCha20Poly1305};
use rand::{rngs::OsRng, RngCore};
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey, RsaPublicKey};

fn _main() {
    let mut key = [0u8; 32];
    let mut nonce = [0u8; 24];
    OsRng.fill_bytes(&mut key);
    OsRng.fill_bytes(&mut nonce);

    let mut rng = OsRng;
    let bits = 2048;
    let priv_key = RsaPrivateKey::new(&mut rng, bits).expect("failed to generate a key");
    let pub_key = RsaPublicKey::from(&priv_key);

    let _ = rsa_encrypt_aes_keys(pub_key, key.to_vec(), nonce.to_vec());

    _encrypt_small_file("src/main.rs", "src/main.rs.enc", &key, &nonce).unwrap();

    let result = _decrypt_small_file_with_rsa_key("src/main.rs.enc", "src/main.rs.dec", priv_key);

    println!("{:?}", result);
}

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

fn _encrypt_small_file(
    filepath: &str,
    dist: &str,
    key: &[u8; 32],
    nonce: &[u8; 24],
) -> Result<(), anyhow::Error> {
    println!("Encrypting small file");

    let cipher = XChaCha20Poly1305::new(key.into());

    let file_data = fs::read(filepath)?;

    let encrypted_file = cipher
        .encrypt(nonce.into(), file_data.as_ref())
        .map_err(|err| anyhow!("Encrypting small file: {}", err))?;

    fs::write(&dist, encrypted_file)?;

    Ok(())
}

fn _decrypt_small_file(
    encrypted_file_path: &str,
    dist: &str,
    key: &[u8; 32],
    nonce: &[u8; 24],
) -> Result<(), anyhow::Error> {
    println!("Decrypting small file");

    let cipher = XChaCha20Poly1305::new(key.into());

    let file_data = fs::read(encrypted_file_path)?;

    let decrypted_file = cipher
        .decrypt(nonce.into(), file_data.as_ref())
        .map_err(|err| anyhow!("Decrypting small file: {}", err))?;

    fs::write(&dist, decrypted_file)?;

    Ok(())
}

fn _decrypt_small_file_with_rsa_key(
    encrypted_file_path: &str,
    dist: &str,
    priv_key: RsaPrivateKey,
) -> Result<(), anyhow::Error> {
    let data = _rsa_decrypt(priv_key);

    let key: [u8; 32] = data[0][0..32].try_into().unwrap();
    let nonce: [u8; 24] = data[2][0..24].try_into().unwrap();

    _decrypt_small_file(encrypted_file_path, dist, &key, &nonce)
}

/// Uses RSA public key to encrypt the AES key and nonce
pub fn rsa_encrypt_aes_keys(
    pub_key: RsaPublicKey,
    key: Vec<u8>,
    nonce: Vec<u8>,
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

fn _rsa_decrypt(priv_key: RsaPrivateKey) -> Vec<Vec<u8>> {
    println!("Decrypting via RSA");

    let mut file = File::open("key.bin").unwrap();
    let mut enc_data = Vec::new();
    file.read_to_end(&mut enc_data).unwrap();

    let mut cursor = 0;
    let mut decrypted_chunks: Vec<Vec<u8>> = Vec::new();

    // Parse length-prefixed blocks until EOF
    while cursor < enc_data.len() {
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
        decrypted_chunks.push(dec);
    }

    println!("Decrypted {} blocks:", decrypted_chunks.len());
    decrypted_chunks
}
