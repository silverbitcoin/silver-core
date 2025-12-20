//! Proof-of-Work types and structures for SilverBitcoin
//!
//! This module defines PoW-specific types including:
//! - Block headers
//! - Work proofs
//! - Mining rewards
//! - Difficulty adjustments

use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::fmt;

// Helper module for serializing fixed-size arrays
mod serde_arrays {
    use serde::{Deserializer, Serializer};

    pub fn serialize<S>(data: &[u8; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(data)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec: Vec<u8> = serde::de::Deserialize::deserialize(deserializer)?;
        let mut array = [0u8; 64];
        if vec.len() != 64 {
            return Err(serde::de::Error::custom("Invalid array length"));
        }
        array.copy_from_slice(&vec);
        Ok(array)
    }
}

/// Block header for PoW mining
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Block version
    pub version: u32,

    /// Parent block hash (SHA-512, 64 bytes)
    #[serde(with = "serde_arrays")]
    pub parent_hash: [u8; 64],

    /// Merkle root of transactions (SHA-512, 64 bytes)
    #[serde(with = "serde_arrays")]
    pub merkle_root: [u8; 64],

    /// Unix timestamp (seconds)
    pub timestamp: u64,

    /// Mining difficulty
    pub difficulty: u64,

    /// Chain ID (0-19 for parallel chains)
    pub chain_id: u32,

    /// Block height
    pub block_height: u64,

    /// Nonce for mining
    pub nonce: u64,

    /// Extra nonce for mining
    pub extra_nonce: u64,
}

impl BlockHeader {
    /// Create a new block header builder
    pub fn builder(
        version: u32,
        parent_hash: [u8; 64],
        merkle_root: [u8; 64],
        timestamp: u64,
    ) -> BlockHeaderBuilder {
        BlockHeaderBuilder {
            version,
            parent_hash,
            merkle_root,
            timestamp,
            difficulty: 1,
            chain_id: 0,
            block_height: 0,
            nonce: 0,
            extra_nonce: 0,
        }
    }



    /// Compute SHA-512 hash of this header
    pub fn hash(&self) -> [u8; 64] {
        let mut hasher = Sha512::new();
        hasher.update(self.version.to_le_bytes());
        hasher.update(self.parent_hash);
        hasher.update(self.merkle_root);
        hasher.update(self.timestamp.to_le_bytes());
        hasher.update(self.difficulty.to_le_bytes());
        hasher.update(self.chain_id.to_le_bytes());
        hasher.update(self.block_height.to_le_bytes());
        hasher.update(self.nonce.to_le_bytes());
        hasher.update(self.extra_nonce.to_le_bytes());

        let mut output = [0u8; 64];
        output.copy_from_slice(&hasher.finalize());
        output
    }

    /// Get header data for hashing (without nonce)
    pub fn get_header_for_hashing(&self) -> Vec<u8> {
        let mut header = Vec::with_capacity(256);
        header.extend_from_slice(&self.version.to_le_bytes());
        header.extend_from_slice(&self.parent_hash);
        header.extend_from_slice(&self.merkle_root);
        header.extend_from_slice(&self.timestamp.to_le_bytes());
        header.extend_from_slice(&self.difficulty.to_le_bytes());
        header.extend_from_slice(&self.chain_id.to_le_bytes());
        header.extend_from_slice(&self.block_height.to_le_bytes());
        header
    }
}

impl fmt::Display for BlockHeader {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "BlockHeader {{ chain: {}, height: {}, difficulty: {}, nonce: {} }}",
            self.chain_id, self.block_height, self.difficulty, self.nonce
        )
    }
}

/// Proof of work - solution to a mining puzzle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkProof {
    /// Work ID
    #[serde(with = "serde_arrays")]
    pub work_id: [u8; 64],

    /// Chain ID
    pub chain_id: u32,

    /// Block height
    pub block_height: u64,

    /// Nonce used
    pub nonce: u64,

    /// Extra nonce used
    pub extra_nonce: u64,

    /// Block hash
    #[serde(with = "serde_arrays")]
    pub block_hash: [u8; 64],

    /// Hash result (SHA-512)
    #[serde(with = "serde_arrays")]
    pub hash_result: [u8; 64],

    /// Timestamp when proof was created
    pub timestamp: u64,

    /// Miner address
    pub miner_address: Vec<u8>,

    /// Difficulty achieved
    pub difficulty_achieved: u64,
}

impl WorkProof {
    /// Create a new work proof builder
    pub fn builder(
        work_id: [u8; 64],
        chain_id: u32,
        block_hash: [u8; 64],
        hash_result: [u8; 64],
        miner_address: Vec<u8>,
    ) -> WorkProofBuilder {
        WorkProofBuilder {
            work_id,
            chain_id,
            block_height: 0,
            nonce: 0,
            extra_nonce: 0,
            block_hash,
            hash_result,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            miner_address,
        }
    }



    /// Verify proof against target
    pub fn verify(&self, target: &[u8; 64]) -> Result<bool> {
        // Hash must be less than or equal to target
        Ok(self.hash_result.as_slice() <= target.as_slice())
    }

    /// Get difficulty from hash (count leading zero bits)
    pub fn get_difficulty_from_hash(hash: &[u8; 64]) -> Result<u64> {
        let mut leading_zeros = 0u64;
        for byte in hash {
            if *byte == 0 {
                leading_zeros += 8;
            } else {
                leading_zeros += byte.leading_zeros() as u64;
                break;
            }
        }

        if leading_zeros >= 64 {
            Ok(u64::MAX)
        } else {
            Ok(1u64 << leading_zeros)
        }
    }

    /// Verify timestamp is not too old
    pub fn verify_timestamp(&self, max_age_seconds: u64) -> Result<bool> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        if now < self.timestamp {
            return Err(Error::InvalidData("Proof timestamp in future".to_string()));
        }

        Ok(now - self.timestamp <= max_age_seconds)
    }
}

impl fmt::Display for WorkProof {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "WorkProof {{ chain: {}, height: {}, difficulty: {}, nonce: {} }}",
            self.chain_id, self.block_height, self.difficulty_achieved, self.nonce
        )
    }
}

/// Mining reward
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningReward {
    /// Block height
    pub block_height: u64,

    /// Base reward (in satoshis)
    pub base_reward: u128,

    /// Transaction fees (in satoshis)
    pub transaction_fees: u128,

    /// Total reward to miner (100% of base + fees)
    pub total_miner_reward: u128,
}

impl MiningReward {
    /// Create a new mining reward
    pub fn new(
        block_height: u64,
        base_reward: u128,
        transaction_fees: u128,
    ) -> Result<Self> {
        let total_miner_reward = base_reward + transaction_fees;

        Ok(Self {
            block_height,
            base_reward,
            transaction_fees,
            total_miner_reward,
        })
    }

    /// Get total reward to miner (100% of block reward)
    pub fn get_miner_reward(&self) -> u128 {
        self.total_miner_reward
    }
}

impl fmt::Display for MiningReward {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "MiningReward {{ height: {}, base: {}, fees: {}, total: {} }}",
            self.block_height, self.base_reward, self.transaction_fees, self.total_miner_reward
        )
    }
}

/// Difficulty adjustment record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyAdjustment {
    /// Chain ID
    pub chain_id: u32,

    /// Current difficulty
    pub current_difficulty: u64,

    /// Previous difficulty
    pub previous_difficulty: u64,

    /// Adjustment factor
    pub adjustment_factor: f64,

    /// Block height when adjusted
    pub block_height: u64,

    /// Adjustment timestamp
    pub timestamp: u64,
}

impl DifficultyAdjustment {
    /// Create a new difficulty adjustment
    pub fn new(
        chain_id: u32,
        current_difficulty: u64,
        previous_difficulty: u64,
        block_height: u64,
        timestamp: u64,
    ) -> Self {
        let adjustment_factor = if previous_difficulty > 0 {
            current_difficulty as f64 / previous_difficulty as f64
        } else {
            1.0
        };

        Self {
            chain_id,
            current_difficulty,
            previous_difficulty,
            adjustment_factor,
            block_height,
            timestamp,
        }
    }
}

impl fmt::Display for DifficultyAdjustment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DifficultyAdjustment {{ chain: {}, {} -> {} (factor: {:.2}x) }}",
            self.chain_id,
            self.previous_difficulty,
            self.current_difficulty,
            self.adjustment_factor
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_header_creation() {
        let header = BlockHeader::new(
            1,
            [1u8; 64],
            [2u8; 64],
            1000,
            1_000_000,
            0,
            100,
            12345,
            0,
        );

        assert!(header.is_ok());
        let header = header.unwrap();
        assert_eq!(header.block_height, 100);
    }

    #[test]
    fn test_block_header_hash() {
        let header = BlockHeader::new(
            1,
            [1u8; 64],
            [2u8; 64],
            1000,
            1_000_000,
            0,
            100,
            12345,
            0,
        )
        .unwrap();

        let hash = header.hash();
        assert_eq!(hash.len(), 64);

        // Verify deterministic
        let hash2 = header.hash();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_mining_reward_creation() {
        let reward = MiningReward::new(100, 50_000_000_000, 1_000_000);
        assert!(reward.is_ok());

        let reward = reward.unwrap();
        assert_eq!(reward.total_miner_reward, 50_001_000_000);
    }

    #[test]
    fn test_difficulty_adjustment() {
        let adj = DifficultyAdjustment::new(0, 2_000_000, 1_000_000, 100, 1000);
        assert_eq!(adj.adjustment_factor, 2.0);
    }
}

/// Builder for BlockHeader - real production-grade builder pattern
pub struct BlockHeaderBuilder {
    version: u32,
    parent_hash: [u8; 64],
    merkle_root: [u8; 64],
    timestamp: u64,
    difficulty: u64,
    chain_id: u32,
    block_height: u64,
    nonce: u64,
    extra_nonce: u64,
}

impl BlockHeaderBuilder {
    /// Set difficulty
    pub fn with_difficulty(mut self, difficulty: u64) -> Self {
        self.difficulty = difficulty;
        self
    }

    /// Set chain ID
    pub fn with_chain_id(mut self, chain_id: u32) -> Self {
        self.chain_id = chain_id;
        self
    }

    /// Set block height
    pub fn with_block_height(mut self, height: u64) -> Self {
        self.block_height = height;
        self
    }

    /// Set nonce
    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    /// Set extra nonce
    pub fn with_extra_nonce(mut self, extra_nonce: u64) -> Self {
        self.extra_nonce = extra_nonce;
        self
    }

    /// Build the block header
    pub fn build(self) -> Result<BlockHeader> {
        if self.difficulty == 0 {
            return Err(Error::InvalidData("Difficulty cannot be zero".to_string()));
        }

        if self.chain_id >= 20 {
            return Err(Error::InvalidData("Invalid chain ID".to_string()));
        }

        Ok(BlockHeader {
            version: self.version,
            parent_hash: self.parent_hash,
            merkle_root: self.merkle_root,
            timestamp: self.timestamp,
            difficulty: self.difficulty,
            chain_id: self.chain_id,
            block_height: self.block_height,
            nonce: self.nonce,
            extra_nonce: self.extra_nonce,
        })
    }
}

/// Builder for WorkProof - real production-grade builder pattern
pub struct WorkProofBuilder {
    work_id: [u8; 64],
    chain_id: u32,
    block_height: u64,
    nonce: u64,
    extra_nonce: u64,
    block_hash: [u8; 64],
    hash_result: [u8; 64],
    timestamp: u64,
    miner_address: Vec<u8>,
}

impl WorkProofBuilder {
    /// Set block height
    pub fn with_block_height(mut self, height: u64) -> Self {
        self.block_height = height;
        self
    }

    /// Set nonce
    pub fn with_nonce(mut self, nonce: u64) -> Self {
        self.nonce = nonce;
        self
    }

    /// Set extra nonce
    pub fn with_extra_nonce(mut self, extra_nonce: u64) -> Self {
        self.extra_nonce = extra_nonce;
        self
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }

    /// Build the work proof
    pub fn build(self) -> Result<WorkProof> {
        if self.miner_address.is_empty() {
            return Err(Error::InvalidData("Miner address cannot be empty".to_string()));
        }

        let difficulty_achieved = WorkProof::get_difficulty_from_hash(&self.hash_result)?;

        Ok(WorkProof {
            work_id: self.work_id,
            chain_id: self.chain_id,
            block_height: self.block_height,
            nonce: self.nonce,
            extra_nonce: self.extra_nonce,
            block_hash: self.block_hash,
            hash_result: self.hash_result,
            timestamp: self.timestamp,
            miner_address: self.miner_address,
            difficulty_achieved,
        })
    }
}
