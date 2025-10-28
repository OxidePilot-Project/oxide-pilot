use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Key, Nonce,
};
use base64::{engine::general_purpose, Engine as _};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: String,
    pub nonce: String,
    pub associated_data: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControl {
    pub user_id: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

pub struct EncryptionManager {
    cipher: Aes256Gcm,
    roles: HashMap<String, Role>,
    access_controls: HashMap<String, AccessControl>,
}

impl EncryptionManager {
    pub fn new(key: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        let key = Key::<Aes256Gcm>::from_slice(key);
        let cipher = Aes256Gcm::new(key);

        let mut roles = HashMap::new();

        // Define default roles
        roles.insert(
            "admin".to_string(),
            Role {
                name: "admin".to_string(),
                permissions: vec![
                    "system.monitor".to_string(),
                    "system.control".to_string(),
                    "rpa.execute".to_string(),
                    "config.modify".to_string(),
                    "data.access".to_string(),
                ],
            },
        );

        roles.insert(
            "user".to_string(),
            Role {
                name: "user".to_string(),
                permissions: vec![
                    "system.monitor".to_string(),
                    "rpa.execute".to_string(),
                    "config.view".to_string(),
                ],
            },
        );

        roles.insert(
            "readonly".to_string(),
            Role {
                name: "readonly".to_string(),
                permissions: vec!["system.monitor".to_string(), "config.view".to_string()],
            },
        );

        Ok(Self {
            cipher,
            roles,
            access_controls: HashMap::new(),
        })
    }

    pub fn encrypt_data(
        &self,
        plaintext: &[u8],
        associated_data: Option<&[u8]>,
    ) -> Result<EncryptedData, Box<dyn std::error::Error>> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext)
            .map_err(|e| format!("Encryption failed: {e:?}"))?;

        Ok(EncryptedData {
            ciphertext: general_purpose::STANDARD.encode(&ciphertext),
            nonce: general_purpose::STANDARD.encode(nonce),
            associated_data: associated_data.map(|data| general_purpose::STANDARD.encode(data)),
        })
    }

    pub fn decrypt_data(
        &self,
        encrypted: &EncryptedData,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let ciphertext = general_purpose::STANDARD.decode(&encrypted.ciphertext)?;
        let nonce = general_purpose::STANDARD.decode(&encrypted.nonce)?;
        let nonce = Nonce::from_slice(&nonce);

        let plaintext = self
            .cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| format!("Decryption failed: {e:?}"))?;
        Ok(plaintext)
    }

    pub fn has_permission(&self, user_id: &str, permission: &str) -> bool {
        if let Some(access_control) = self.access_controls.get(user_id) {
            if access_control.permissions.contains(&permission.to_string()) {
                return true;
            }

            // Check role-based permissions
            for role_name in &access_control.roles {
                if let Some(role) = self.roles.get(role_name) {
                    if role.permissions.contains(&permission.to_string()) {
                        return true;
                    }
                }
            }
        }

        false
    }

    pub fn add_user(&mut self, user_id: String, roles: Vec<String>) {
        let mut permissions = Vec::new();

        for role_name in &roles {
            if let Some(role) = self.roles.get(role_name) {
                permissions.extend(role.permissions.clone());
            }
        }

        permissions.dedup();

        self.access_controls.insert(
            user_id.clone(),
            AccessControl {
                user_id,
                roles,
                permissions,
            },
        );
    }

    pub fn get_user_permissions(&self, user_id: &str) -> Option<&AccessControl> {
        self.access_controls.get(user_id)
    }

    pub fn generate_key() -> Vec<u8> {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        key.to_vec()
    }
}
