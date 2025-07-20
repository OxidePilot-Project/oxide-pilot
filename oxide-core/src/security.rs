use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};
use rand::{thread_rng, RngCore};
use log::{info, error};

pub fn encrypt_data(data: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Failed to create cipher: {:?}", e))?;
    let mut nonce_bytes = [0u8; 12];
    thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher.encrypt(nonce, data).map_err(|e| format!("Encryption failed: {:?}", e))?;

    let mut result = Vec::with_capacity(nonce_bytes.len() + ciphertext.len());
    result.extend_from_slice(&nonce_bytes);
    result.extend_from_slice(&ciphertext);

    Ok(result)
}

pub fn decrypt_data(encrypted_data: &[u8], key: &[u8]) -> Result<Vec<u8>, String> {
    if encrypted_data.len() < 12 {
        return Err("Encrypted data too short".to_string());
    }

    let cipher = Aes256Gcm::new_from_slice(key).map_err(|e| format!("Failed to create cipher: {:?}", e))?;
    let nonce = Nonce::from_slice(&encrypted_data[..12]);
    let ciphertext = &encrypted_data[12..];

    let plaintext = cipher.decrypt(nonce, ciphertext).map_err(|e| format!("Decryption failed: {:?}", e))?;
    Ok(plaintext)
}

pub fn generate_key() -> Vec<u8> {
    let mut key = [0u8; 32];
    thread_rng().fill_bytes(&mut key);
    key.to_vec()
}
