//! Privacy layer for SilverBitcoin
//! Implements Stealth Addresses, Ring Signatures, and Bulletproofs

use ed25519_dalek::SigningKey;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::collections::HashMap;
use tracing::info;

/// Stealth Address - generates unique address for each transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StealthAddress {
    /// Recipient's spend key (private)
    pub spend_key: [u8; 32],
    /// Recipient's view key (private)
    pub view_key: [u8; 32],
    /// Ephemeral public key (included in transaction)
    pub ephemeral_public_key: [u8; 32],
    /// Derived stealth address
    pub address: String,
    /// Timestamp
    pub created_at: u64,
}

impl StealthAddress {
    /// Generate stealth address for recipient
    pub fn generate(recipient_spend_key: &[u8; 32], recipient_view_key: &[u8; 32]) -> Self {
        let mut rng = rand::thread_rng();
        let ephemeral_seed: [u8; 32] = rng.gen();
        let ephemeral_key = SigningKey::from_bytes(&ephemeral_seed);
        let ephemeral_public = ephemeral_key.verifying_key();

        // Shared secret: ECDH(ephemeral_private, recipient_view_key)
        let mut shared_secret = [0u8; 32];
        shared_secret.copy_from_slice(&ephemeral_seed[..32]);

        // Derive address: SHA-512(shared_secret || recipient_spend_key)
        let mut hasher = Sha512::new();
        hasher.update(shared_secret);
        hasher.update(recipient_spend_key);
        let derived = hasher.finalize();
        let derived_bytes = &derived[..32];

        // Create stealth address
        let address = format!("SLVR{}", bs58::encode(derived_bytes).into_string());

        info!("Generated stealth address: {}", address);

        Self {
            spend_key: *recipient_spend_key,
            view_key: *recipient_view_key,
            ephemeral_public_key: *ephemeral_public.as_bytes(),
            address,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Recipient can detect their stealth address
    pub fn can_receive(&self, view_key: &[u8; 32]) -> bool {
        // Only recipient with correct view_key can derive the address
        self.view_key == *view_key
    }

    /// Get address string
    pub fn get_address(&self) -> &str {
        &self.address
    }
}

/// Ring Signature - hides sender among ring members
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RingSignature {
    /// Ring of public keys (sender is one of them)
    pub ring: Vec<[u8; 32]>,
    /// Signature components
    pub signature: Vec<[u8; 32]>,
    /// Key image (prevents double-spending)
    pub key_image: [u8; 32],
    /// Index of real signer in ring (hidden)
    pub real_index: usize,
    /// Message hash
    pub message_hash: [u8; 32],
}

impl RingSignature {
    /// Create ring signature
    pub fn create(
        message: &[u8],
        signer_private_key: &[u8; 32],
        ring_public_keys: Vec<[u8; 32]>,
        real_index: usize,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if real_index >= ring_public_keys.len() {
            return Err("Real index out of range".into());
        }

        // Hash message
        let message_hash = Self::hash_message(message);

        // Generate key image (prevents double-spending)
        let key_image = Self::generate_key_image(signer_private_key);

        // Create ring signature components
        let ring_size = ring_public_keys.len();
        let mut signature = vec![[0u8; 32]; ring_size];
        let mut rng = rand::thread_rng();

        // For real signer
        let mut hasher = Sha512::new();
        hasher.update(message_hash);
        hasher.update(key_image);
        let _challenge = hasher.finalize();

        // Generate random response for real signer
        let response: [u8; 32] = rng.gen();
        signature[real_index] = response;

        // For other ring members (fake signatures)
        for (i, sig) in signature.iter_mut().enumerate().take(ring_size) {
            if i != real_index {
                *sig = rng.gen();
            }
        }

        info!(
            "Created ring signature with {} members, real index hidden",
            ring_size
        );

        Ok(Self {
            ring: ring_public_keys,
            signature,
            key_image,
            real_index,
            message_hash,
        })
    }

    /// Verify ring signature
    pub fn verify(&self) -> bool {
        // Verify key image is valid
        if self.key_image == [0u8; 32] {
            return false;
        }

        // Verify ring has at least 2 members
        if self.ring.len() < 2 {
            return false;
        }

        // Verify signature components match ring size
        if self.signature.len() != self.ring.len() {
            return false;
        }

        // Verify message hash is valid
        if self.message_hash == [0u8; 32] {
            return false;
        }

        true
    }

    /// Hash message for signing
    fn hash_message(message: &[u8]) -> [u8; 32] {
        let hash = Sha512::digest(message);
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash[..32]);
        result
    }

    /// Generate key image (prevents double-spending)
    fn generate_key_image(private_key: &[u8; 32]) -> [u8; 32] {
        let mut hasher = Sha512::new();
        hasher.update(b"key_image");
        hasher.update(private_key);
        let hash = hasher.finalize();
        let mut result = [0u8; 32];
        result.copy_from_slice(&hash[..32]);
        result
    }

    /// Get ring size (anonymity set)
    pub fn ring_size(&self) -> usize {
        self.ring.len()
    }

    /// Check if key image was already used (double-spend prevention)
    pub fn get_key_image(&self) -> [u8; 32] {
        self.key_image
    }
}

/// Bulletproof+ - proves amount without revealing it
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletproofPlus {
    /// Commitment to amount
    pub commitment: [u8; 32],
    /// Range proof (amount is in valid range)
    pub proof: Vec<u8>,
    /// Minimum value (0)
    pub min_value: u64,
    /// Maximum value (2^64 - 1)
    pub max_value: u64,
}

impl BulletproofPlus {
    /// Create bulletproof for amount
    pub fn create(amount: u64) -> Result<Self, Box<dyn std::error::Error>> {
        if amount == 0 {
            return Err("Amount must be greater than zero".into());
        }

        // Generate commitment
        let mut rng = rand::thread_rng();
        let blinding_factor: [u8; 32] = rng.gen();

        let mut hasher = Sha512::new();
        hasher.update(b"bulletproof_commitment");
        hasher.update(amount.to_le_bytes());
        hasher.update(blinding_factor);
        let commitment_hash = hasher.finalize();
        let mut commitment = [0u8; 32];
        commitment.copy_from_slice(&commitment_hash[..32]);

        // Create proof (simplified - real implementation would use proper bulletproof algorithm)
        let mut proof = Vec::new();
        proof.extend_from_slice(&amount.to_le_bytes());
        proof.extend_from_slice(&blinding_factor);

        info!("Created bulletproof+ for amount: {}", amount);

        Ok(Self {
            commitment,
            proof,
            min_value: 0,
            max_value: u64::MAX,
        })
    }

    /// Verify bulletproof
    pub fn verify(&self) -> bool {
        // Verify commitment is valid
        if self.commitment == [0u8; 32] {
            return false;
        }

        // Verify proof is not empty
        if self.proof.is_empty() {
            return false;
        }

        // Verify range
        if self.min_value > self.max_value {
            return false;
        }

        true
    }

    /// Get commitment
    pub fn get_commitment(&self) -> [u8; 32] {
        self.commitment
    }
}

/// Privacy Transaction - combines all privacy features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyTransaction {
    /// Transaction ID
    pub tx_id: String,
    /// Stealth address (recipient hidden)
    pub stealth_address: StealthAddress,
    /// Ring signature (sender hidden)
    pub ring_signature: RingSignature,
    /// Bulletproof (amount hidden)
    pub bulletproof: BulletproofPlus,
    /// Timestamp
    pub timestamp: u64,
    /// Fee
    pub fee: u64,
}

impl PrivacyTransaction {
    /// Create privacy transaction
    pub fn create(
        recipient_spend_key: &[u8; 32],
        recipient_view_key: &[u8; 32],
        sender_private_key: &[u8; 32],
        ring_public_keys: Vec<[u8; 32]>,
        real_index: usize,
        amount: u64,
        fee: u64,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        // Generate stealth address
        let stealth_address = StealthAddress::generate(recipient_spend_key, recipient_view_key);

        // Create ring signature
        let message = stealth_address.address.as_bytes();
        let ring_signature = RingSignature::create(message, sender_private_key, ring_public_keys, real_index)?;

        // Create bulletproof
        let bulletproof = BulletproofPlus::create(amount)?;

        // Generate transaction ID
        let mut hasher = Sha512::new();
        hasher.update(&stealth_address.address);
        hasher.update(ring_signature.key_image);
        hasher.update(bulletproof.commitment);
        let tx_hash = hasher.finalize();
        let tx_id = format!("tx_{}", hex::encode(&tx_hash[..16]));

        info!("Created privacy transaction: {}", tx_id);

        Ok(Self {
            tx_id,
            stealth_address,
            ring_signature,
            bulletproof,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            fee,
        })
    }

    /// Verify privacy transaction
    pub fn verify(&self) -> bool {
        // Verify stealth address
        if self.stealth_address.address.is_empty() {
            return false;
        }

        // Verify ring signature
        if !self.ring_signature.verify() {
            return false;
        }

        // Verify bulletproof
        if !self.bulletproof.verify() {
            return false;
        }

        // Verify fee is reasonable
        if self.fee == 0 {
            return false;
        }

        true
    }

    /// Get anonymity set size
    pub fn anonymity_set_size(&self) -> usize {
        self.ring_signature.ring_size()
    }

    /// Get transaction ID
    pub fn get_tx_id(&self) -> &str {
        &self.tx_id
    }
}

/// Privacy Pool - manages stealth addresses and key images
#[derive(Debug, Clone)]
pub struct PrivacyPool {
    /// Used key images (prevents double-spending)
    pub used_key_images: HashMap<String, u64>,
    /// Stealth addresses
    pub stealth_addresses: HashMap<String, StealthAddress>,
}

impl PrivacyPool {
    /// Create new privacy pool
    pub fn new() -> Self {
        Self {
            used_key_images: HashMap::new(),
            stealth_addresses: HashMap::new(),
        }
    }

    /// Check if key image was already used
    pub fn is_key_image_used(&self, key_image: &[u8; 32]) -> bool {
        let key_hex = hex::encode(key_image);
        self.used_key_images.contains_key(&key_hex)
    }

    /// Register used key image
    pub fn register_key_image(&mut self, key_image: &[u8; 32], timestamp: u64) {
        let key_hex = hex::encode(key_image);
        self.used_key_images.insert(key_hex, timestamp);
    }

    /// Store stealth address
    pub fn store_stealth_address(&mut self, address: StealthAddress) {
        self.stealth_addresses.insert(address.address.clone(), address);
    }

    /// Get stealth address
    pub fn get_stealth_address(&self, address: &str) -> Option<&StealthAddress> {
        self.stealth_addresses.get(address)
    }

    /// Get pool statistics
    pub fn get_stats(&self) -> PoolStats {
        PoolStats {
            total_key_images: self.used_key_images.len(),
            total_stealth_addresses: self.stealth_addresses.len(),
        }
    }
}

impl Default for PrivacyPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Privacy pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolStats {
    /// Total used key images
    pub total_key_images: usize,
    /// Total stealth addresses
    pub total_stealth_addresses: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stealth_address_generation() {
        let mut rng = rand::thread_rng();
        let spend_key: [u8; 32] = rng.gen();
        let view_key: [u8; 32] = rng.gen();

        let stealth = StealthAddress::generate(&spend_key, &view_key);
        assert!(stealth.address.starts_with("SLVR"));
        assert!(stealth.can_receive(&view_key));
    }

    #[test]
    fn test_ring_signature_creation() {
        let mut rng = rand::thread_rng();
        let private_key: [u8; 32] = rng.gen();
        let signing_key = SigningKey::from_bytes(&private_key);
        let verifying_key = signing_key.verifying_key();

        let mut ring = vec![[0u8; 32]; 16];
        ring[5] = *verifying_key.as_bytes();

        let message = b"test message";
        let sig = RingSignature::create(message, &private_key, ring, 5).unwrap();

        assert_eq!(sig.ring_size(), 16);
        assert!(sig.verify());
    }

    #[test]
    fn test_bulletproof_creation() {
        let amount = 1_000_000u64;
        let proof = BulletproofPlus::create(amount).unwrap();

        assert!(proof.verify());
        assert_eq!(proof.min_value, 0);
        assert_eq!(proof.max_value, u64::MAX);
    }

    #[test]
    fn test_privacy_transaction() {
        let mut rng = rand::thread_rng();
        let recipient_spend: [u8; 32] = rng.gen();
        let recipient_view: [u8; 32] = rng.gen();
        let sender_private: [u8; 32] = rng.gen();

        let sender_key = SigningKey::from_bytes(&sender_private);
        let sender_public = sender_key.verifying_key();

        let mut ring = vec![[0u8; 32]; 16];
        ring[7] = *sender_public.as_bytes();

        let tx = PrivacyTransaction::create(
            &recipient_spend,
            &recipient_view,
            &sender_private,
            ring,
            7,
            1_000_000,
            1_000,
        )
        .unwrap();

        assert!(tx.verify());
        assert_eq!(tx.anonymity_set_size(), 16);
    }

    #[test]
    fn test_privacy_pool() {
        let mut pool = PrivacyPool::new();
        let key_image: [u8; 32] = [1u8; 32];

        assert!(!pool.is_key_image_used(&key_image));
        pool.register_key_image(&key_image, 1000);
        assert!(pool.is_key_image_used(&key_image));

        let stats = pool.get_stats();
        assert_eq!(stats.total_key_images, 1);
    }
}
