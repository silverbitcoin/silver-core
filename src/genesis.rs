//! Genesis block initialization for SilverBitcoin blockchain
//! Handles creation of the initial block and chain state

use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::info;

/// Genesis block configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Chain ID (0 for mainnet, 1+ for testnets)
    pub chain_id: u32,
    /// Genesis block timestamp
    pub timestamp: u64,
    /// Initial difficulty
    pub initial_difficulty: u64,
    /// Genesis block message
    pub message: String,
}

impl GenesisConfig {
    /// Create mainnet genesis configuration
    pub fn mainnet() -> Self {
        let timestamp = 1703001600; // December 20, 2023 00:00:00 UTC

        Self {
            chain_id: 0,
            timestamp,
            initial_difficulty: 1_000_000,
            message: "SilverBitcoin Genesis Block - Quantum-Resistant Blockchain".to_string(),
        }
    }

    /// Create testnet genesis configuration
    pub fn testnet() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            chain_id: 1,
            timestamp,
            initial_difficulty: 100_000,
            message: "SilverBitcoin Testnet Genesis Block".to_string(),
        }
    }

    /// Create devnet genesis configuration
    pub fn devnet() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            chain_id: 2,
            timestamp,
            initial_difficulty: 10_000,
            message: "SilverBitcoin Devnet Genesis Block".to_string(),
        }
    }

    /// Create custom genesis configuration
    pub fn custom(chain_id: u32, difficulty: u64, message: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            chain_id,
            timestamp,
            initial_difficulty: difficulty,
            message,
        }
    }
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self::mainnet()
    }
}

/// Genesis block structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisBlock {
    /// Chain ID
    pub chain_id: u32,
    /// Block height (always 0)
    pub height: u64,
    /// Block timestamp
    pub timestamp: u64,
    /// Genesis message
    pub message: String,
    /// Initial difficulty
    pub difficulty: u64,
    /// Genesis block hash
    pub hash: Vec<u8>,
    /// Merkle root
    pub merkle_root: Vec<u8>,
    /// Previous block hash (all zeros for genesis)
    pub previous_hash: Vec<u8>,
}

impl GenesisBlock {
    /// Create a new genesis block
    pub fn new(config: GenesisConfig) -> Self {
        let message_bytes = config.message.as_bytes();
        let mut hash_input = Vec::new();

        // Build hash input
        hash_input.extend_from_slice(&config.chain_id.to_le_bytes());
        hash_input.extend_from_slice(&0u64.to_le_bytes()); // height = 0
        hash_input.extend_from_slice(&config.timestamp.to_le_bytes());
        hash_input.extend_from_slice(message_bytes);
        hash_input.extend_from_slice(&config.initial_difficulty.to_le_bytes());

        // Calculate hash using SHA-512
        let hash = sha2::Sha512::digest(&hash_input).to_vec();

        // Merkle root is the hash of the message using SHA-512
        let merkle_root = sha2::Sha512::digest(message_bytes).to_vec();

        // Previous hash is all zeros for genesis
        let previous_hash = vec![0u8; 64]; // SHA-512 produces 64 bytes

        info!(
            "Creating genesis block for chain {} with difficulty {}",
            config.chain_id, config.initial_difficulty
        );

        Self {
            chain_id: config.chain_id,
            height: 0,
            timestamp: config.timestamp,
            message: config.message,
            difficulty: config.initial_difficulty,
            hash,
            merkle_root,
            previous_hash,
        }
    }

    /// Get genesis block for mainnet
    pub fn mainnet() -> Self {
        Self::new(GenesisConfig::mainnet())
    }

    /// Get genesis block for testnet
    pub fn testnet() -> Self {
        Self::new(GenesisConfig::testnet())
    }

    /// Get genesis block for devnet
    pub fn devnet() -> Self {
        Self::new(GenesisConfig::devnet())
    }

    /// Verify genesis block integrity
    pub fn verify(&self) -> bool {
        // Verify hash
        let message_bytes = self.message.as_bytes();
        let mut hash_input = Vec::new();

        hash_input.extend_from_slice(&self.chain_id.to_le_bytes());
        hash_input.extend_from_slice(&self.height.to_le_bytes());
        hash_input.extend_from_slice(&self.timestamp.to_le_bytes());
        hash_input.extend_from_slice(message_bytes);
        hash_input.extend_from_slice(&self.difficulty.to_le_bytes());

        let calculated_hash = sha2::Sha512::digest(&hash_input).to_vec();

        if calculated_hash != self.hash {
            return false;
        }

        // Verify merkle root
        let calculated_merkle = sha2::Sha512::digest(message_bytes).to_vec();
        if calculated_merkle != self.merkle_root {
            return false;
        }

        // Verify previous hash is all zeros (SHA-512 = 64 bytes)
        if self.previous_hash != vec![0u8; 64] {
            return false;
        }

        // Verify height is 0
        if self.height != 0 {
            return false;
        }

        true
    }

    /// Get genesis block hash as hex string
    pub fn hash_hex(&self) -> String {
        hex::encode(&self.hash)
    }

    /// Get genesis block hash as hex string (short form)
    pub fn hash_short(&self) -> String {
        let hex = self.hash_hex();
        format!("{}...{}", &hex[..8], &hex[hex.len() - 8..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_config_mainnet() {
        let config = GenesisConfig::mainnet();
        assert_eq!(config.chain_id, 0);
        assert_eq!(config.initial_difficulty, 1_000_000);
    }

    #[test]
    fn test_genesis_config_testnet() {
        let config = GenesisConfig::testnet();
        assert_eq!(config.chain_id, 1);
        assert_eq!(config.initial_difficulty, 100_000);
    }

    #[test]
    fn test_genesis_config_devnet() {
        let config = GenesisConfig::devnet();
        assert_eq!(config.chain_id, 2);
        assert_eq!(config.initial_difficulty, 10_000);
    }

    #[test]
    fn test_genesis_block_creation() {
        let genesis = GenesisBlock::mainnet();
        assert_eq!(genesis.chain_id, 0);
        assert_eq!(genesis.height, 0);
        assert!(genesis.verify());
    }

    #[test]
    fn test_genesis_block_verify() {
        let genesis = GenesisBlock::testnet();
        assert!(genesis.verify());
    }

    #[test]
    fn test_genesis_block_hash() {
        let genesis = GenesisBlock::mainnet();
        let hash_hex = genesis.hash_hex();
        assert_eq!(hash_hex.len(), 128); // 64 bytes (512-bit Blake3) = 128 hex chars
    }
}
