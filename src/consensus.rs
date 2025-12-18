//! Consensus data structures
//!
//! This module defines the consensus-related types for SilverBitcoin's
//! Mercury Protocol and Cascade mempool implementation.

use crate::{
    Error, PublicKey, Result, Signature, SilverAddress, SnapshotDigest, StateDigest, Transaction,
    TransactionDigest,
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

// Macro to implement Serialize/Deserialize for 64-byte array wrappers
macro_rules! impl_serde_64 {
    ($type:ident) => {
        impl Serialize for $type {
            fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
            where
                S: Serializer,
            {
                serializer.serialize_bytes(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $type {
            fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> serde::de::Visitor<'de> for Visitor {
                    type Value = $type;

                    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                        formatter.write_str("a 64-byte array")
                    }

                    fn visit_bytes<E>(self, v: &[u8]) -> std::result::Result<Self::Value, E>
                    where
                        E: serde::de::Error,
                    {
                        if v.len() != 64 {
                            return Err(E::custom(format!("expected 64 bytes, got {}", v.len())));
                        }
                        let mut arr = [0u8; 64];
                        arr.copy_from_slice(v);
                        Ok($type(arr))
                    }

                    fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
                    where
                        A: serde::de::SeqAccess<'de>,
                    {
                        let mut arr = [0u8; 64];
                        for i in 0..64 {
                            arr[i] = seq
                                .next_element()?
                                .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                        }
                        Ok($type(arr))
                    }
                }

                deserializer.deserialize_bytes(Visitor)
            }
        }
    };
}

/// Transaction batch ID (512-bit Blake3 hash)
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BatchID(pub [u8; 64]);

impl_serde_64!(BatchID);

impl BatchID {
    /// Create a new batch ID
    pub const fn new(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// Get bytes as slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Compute batch ID from batch data
    pub fn compute(
        transactions: &[Transaction],
        author: &SilverAddress,
        timestamp: u64,
        previous_batches: &[BatchID],
    ) -> Self {
        let mut hasher = blake3::Hasher::new();

        // Hash author
        hasher.update(author.as_bytes());

        // Hash timestamp
        hasher.update(&timestamp.to_le_bytes());

        // Hash previous batches
        for batch_id in previous_batches {
            hasher.update(batch_id.as_bytes());
        }

        // Hash transactions
        for tx in transactions {
            hasher.update(tx.digest().as_bytes());
        }

        let mut output = [0u8; 64];
        hasher.finalize_xof().fill(&mut output);
        Self(output)
    }
}

impl fmt::Debug for BatchID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BatchID({}...)", &self.to_hex()[..16])
    }
}

impl fmt::Display for BatchID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_hex()[..16])
    }
}

/// Transaction batch for Cascade mempool
///
/// Batches are created by validator workers and form a flow graph
/// through cryptographic links to previous batches.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBatch {
    /// Unique batch identifier
    pub batch_id: BatchID,

    /// Transactions in this batch
    pub transactions: Vec<Transaction>,

    /// Validator that created this batch
    pub author: ValidatorID,

    /// Unix timestamp when batch was created (milliseconds)
    pub timestamp: u64,

    /// Previous batches this batch depends on (flow graph links)
    pub previous_batches: Vec<BatchID>,

    /// Signature from the author
    pub author_signature: Signature,
}

impl TransactionBatch {
    /// Create a new transaction batch
    pub fn new(
        transactions: Vec<Transaction>,
        author: ValidatorID,
        timestamp: u64,
        previous_batches: Vec<BatchID>,
        author_signature: Signature,
    ) -> Result<Self> {
        // Validate batch size constraints
        if transactions.is_empty() {
            return Err(Error::InvalidData(
                "Batch must contain at least one transaction".to_string(),
            ));
        }

        if transactions.len() > 500 {
            return Err(Error::InvalidData(format!(
                "Batch cannot contain more than 500 transactions, got {}",
                transactions.len()
            )));
        }

        // Calculate total size
        let total_size: usize = transactions.iter().map(|tx| tx.size_bytes()).sum();
        if total_size > 512 * 1024 {
            return Err(Error::InvalidData(format!(
                "Batch size cannot exceed 512KB, got {} bytes",
                total_size
            )));
        }

        // Compute batch ID
        let batch_id =
            BatchID::compute(&transactions, &author.address, timestamp, &previous_batches);

        Ok(Self {
            batch_id,
            transactions,
            author,
            timestamp,
            previous_batches,
            author_signature,
        })
    }

    /// Get the number of transactions in this batch
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

    /// Get the total size of this batch in bytes
    pub fn size_bytes(&self) -> usize {
        self.transactions.iter().map(|tx| tx.size_bytes()).sum()
    }

    /// Validate batch structure
    pub fn validate(&self) -> Result<()> {
        if self.transactions.is_empty() {
            return Err(Error::InvalidData("Batch cannot be empty".to_string()));
        }

        if self.transactions.len() > 500 {
            return Err(Error::InvalidData(format!(
                "Batch has too many transactions: {}",
                self.transactions.len()
            )));
        }

        let total_size = self.size_bytes();
        if total_size > 512 * 1024 {
            return Err(Error::InvalidData(format!(
                "Batch size exceeds 512KB: {} bytes",
                total_size
            )));
        }

        // Verify batch ID
        let computed_id = BatchID::compute(
            &self.transactions,
            &self.author.address,
            self.timestamp,
            &self.previous_batches,
        );
        if computed_id != self.batch_id {
            return Err(Error::InvalidData("Batch ID mismatch".to_string()));
        }

        Ok(())
    }
}

impl fmt::Display for TransactionBatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Batch {{ id: {}, txs: {}, size: {} bytes, author: {} }}",
            self.batch_id,
            self.transaction_count(),
            self.size_bytes(),
            self.author.address
        )
    }
}

/// Validator signature on a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSignature {
    /// Validator who signed
    pub validator: ValidatorID,

    /// Signature bytes
    pub signature: Signature,

    /// Timestamp when signature was created (for expiration tracking)
    pub timestamp: std::time::SystemTime,
}

impl ValidatorSignature {
    /// Create a new validator signature
    pub fn new(validator: ValidatorID, signature: Signature) -> Self {
        Self {
            validator,
            signature,
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Create a validator signature with explicit timestamp
    pub fn with_timestamp(
        validator: ValidatorID,
        signature: Signature,
        timestamp: std::time::SystemTime,
    ) -> Self {
        Self {
            validator,
            signature,
            timestamp,
        }
    }
}

/// Batch certificate proving 2/3+ stake agreement
///
/// A certificate is created when a batch receives signatures from
/// validators representing more than 2/3 of the total stake weight.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Certificate {
    /// Batch this certificate is for
    pub batch_id: BatchID,

    /// Validator signatures
    pub signatures: Vec<ValidatorSignature>,

    /// Total stake weight of signers
    pub stake_weight: u64,

    /// Timestamp when certificate was created
    pub timestamp: u64,
}

impl Certificate {
    /// Create a new certificate
    pub fn new(
        batch_id: BatchID,
        signatures: Vec<ValidatorSignature>,
        stake_weight: u64,
        timestamp: u64,
    ) -> Self {
        Self {
            batch_id,
            signatures,
            stake_weight,
            timestamp,
        }
    }

    /// Check if this certificate has sufficient stake weight
    pub fn has_quorum(&self, total_stake: u64) -> bool {
        // Require 2/3+ stake weight
        self.stake_weight * 3 > total_stake * 2
    }

    /// Get the number of validator signatures
    pub fn signature_count(&self) -> usize {
        self.signatures.len()
    }

    /// Validate certificate structure
    pub fn validate(&self, total_stake: u64) -> Result<()> {
        if self.signatures.is_empty() {
            return Err(Error::InvalidData(
                "Certificate must have at least one signature".to_string(),
            ));
        }

        if !self.has_quorum(total_stake) {
            return Err(Error::InvalidData(format!(
                "Certificate does not have quorum: {} / {} stake",
                self.stake_weight, total_stake
            )));
        }

        Ok(())
    }
}

impl fmt::Display for Certificate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Certificate {{ batch: {}, signatures: {}, stake: {} }}",
            self.batch_id,
            self.signature_count(),
            self.stake_weight
        )
    }
}

/// Snapshot sequence number
pub type SnapshotSequenceNumber = u64;

/// Cycle ID for validator set epochs
pub type CycleID = u64;

/// Snapshot (checkpoint) representing finalized state
///
/// Snapshots are created at regular intervals (every 480ms) and represent
/// a finalized point in the blockchain with validator consensus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Monotonically increasing sequence number
    pub sequence_number: SnapshotSequenceNumber,

    /// Unix timestamp (milliseconds)
    pub timestamp: u64,

    /// Digest of previous snapshot
    pub previous_digest: SnapshotDigest,

    /// Root hash of the state tree
    pub root_state_digest: StateDigest,

    /// Transaction digests included in this snapshot
    pub transactions: Vec<TransactionDigest>,

    /// Cycle ID (validator set epoch)
    pub cycle: CycleID,

    /// Validator signatures on this snapshot
    pub validator_signatures: Vec<ValidatorSignature>,

    /// Total stake weight of signers
    pub stake_weight: u64,

    /// Snapshot digest (hash of all fields)
    pub digest: SnapshotDigest,

    /// Validator address that created this snapshot
    pub validator: SilverAddress,
}

impl Snapshot {
    /// Create a new snapshot
    pub fn new(
        sequence_number: SnapshotSequenceNumber,
        timestamp: u64,
        previous_digest: SnapshotDigest,
        root_state_digest: StateDigest,
        transactions: Vec<TransactionDigest>,
        cycle: CycleID,
        validator_signatures: Vec<ValidatorSignature>,
        stake_weight: u64,
        validator: SilverAddress,
    ) -> Self {
        let mut snapshot = Self {
            sequence_number,
            timestamp,
            previous_digest,
            root_state_digest,
            transactions,
            cycle,
            validator_signatures,
            stake_weight,
            digest: SnapshotDigest::new([0u8; 64]),
            validator,
        };

        // Compute digest
        snapshot.digest = snapshot.compute_digest();
        snapshot
    }

    /// Compute the digest of this snapshot
    pub fn compute_digest(&self) -> SnapshotDigest {
        let mut hasher = blake3::Hasher::new();

        hasher.update(&self.sequence_number.to_le_bytes());
        hasher.update(&self.timestamp.to_le_bytes());
        hasher.update(self.previous_digest.as_bytes());
        hasher.update(self.root_state_digest.as_bytes());

        for tx_digest in &self.transactions {
            hasher.update(tx_digest.as_bytes());
        }

        hasher.update(&self.cycle.to_le_bytes());

        let mut output = [0u8; 64];
        hasher.finalize_xof().fill(&mut output);
        SnapshotDigest::new(output)
    }

    /// Check if this snapshot has sufficient stake weight
    pub fn has_quorum(&self, total_stake: u64) -> bool {
        self.stake_weight * 3 > total_stake * 2
    }

    /// Get the number of transactions in this snapshot
    pub fn transaction_count(&self) -> usize {
        self.transactions.len()
    }

    /// Validate snapshot structure
    pub fn validate(&self, total_stake: u64) -> Result<()> {
        // Verify digest
        let computed_digest = self.compute_digest();
        if computed_digest != self.digest {
            return Err(Error::InvalidData("Snapshot digest mismatch".to_string()));
        }

        // Verify quorum
        if !self.has_quorum(total_stake) {
            return Err(Error::InvalidData(format!(
                "Snapshot does not have quorum: {} / {} stake",
                self.stake_weight, total_stake
            )));
        }

        // Verify transaction limit
        if self.transactions.len() > 1000 {
            return Err(Error::InvalidData(format!(
                "Snapshot has too many transactions: {}",
                self.transactions.len()
            )));
        }

        Ok(())
    }

    /// Check if this is the genesis snapshot
    pub fn is_genesis(&self) -> bool {
        self.sequence_number == 0
    }
}

impl fmt::Display for Snapshot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Snapshot {{ seq: {}, txs: {}, cycle: {}, stake: {} }}",
            self.sequence_number,
            self.transaction_count(),
            self.cycle,
            self.stake_weight
        )
    }
}

/// Validator identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ValidatorID {
    /// Validator's SilverBitcoin address
    pub address: SilverAddress,
}

impl ValidatorID {
    /// Create a new validator ID
    pub fn new(address: SilverAddress) -> Self {
        Self { address }
    }
}

impl fmt::Display for ValidatorID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.address)
    }
}

/// Validator metadata and configuration
///
/// Contains all information about a validator including keys,
/// network addresses, and stake amount.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorMetadata {
    /// Validator's SilverBitcoin address
    pub silver_address: SilverAddress,

    /// Protocol public key (for consensus signing)
    pub protocol_pubkey: PublicKey,

    /// Network public key (for P2P authentication)
    pub network_pubkey: PublicKey,

    /// Worker public key (for batch creation)
    pub worker_pubkey: PublicKey,

    /// Amount of SBTC staked (minimum 1,000,000)
    pub stake_amount: u64,

    /// Network address for RPC/API
    pub network_address: String,

    /// P2P address for validator communication
    pub p2p_address: String,

    /// Validator name (optional)
    pub name: Option<String>,

    /// Validator description (optional)
    pub description: Option<String>,

    /// Commission rate (basis points, 0-10000)
    pub commission_rate: u16,
}

impl ValidatorMetadata {
    /// Create new validator metadata
    pub fn new(
        silver_address: SilverAddress,
        protocol_pubkey: PublicKey,
        network_pubkey: PublicKey,
        worker_pubkey: PublicKey,
        stake_amount: u64,
        network_address: String,
        p2p_address: String,
    ) -> Result<Self> {
        if stake_amount < 1_000_000 {
            return Err(Error::InvalidData(format!(
                "Validator stake must be at least 1,000,000 SBTC, got {}",
                stake_amount
            )));
        }

        Ok(Self {
            silver_address,
            protocol_pubkey,
            network_pubkey,
            worker_pubkey,
            stake_amount,
            network_address,
            p2p_address,
            name: None,
            description: None,
            commission_rate: 0,
        })
    }

    /// Set validator name
    pub fn with_name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    /// Set validator description
    pub fn with_description(mut self, description: String) -> Self {
        self.description = Some(description);
        self
    }

    /// Set commission rate (basis points)
    pub fn with_commission_rate(mut self, rate: u16) -> Result<Self> {
        if rate > 10000 {
            return Err(Error::InvalidData(format!(
                "Commission rate cannot exceed 10000 basis points (100%), got {}",
                rate
            )));
        }
        self.commission_rate = rate;
        Ok(self)
    }

    /// Get validator ID
    pub fn id(&self) -> ValidatorID {
        ValidatorID::new(self.silver_address)
    }

    /// Validate validator metadata
    pub fn validate(&self) -> Result<()> {
        if self.stake_amount < 1_000_000 {
            return Err(Error::InvalidData(format!(
                "Insufficient stake: {} SBTC (minimum 1,000,000)",
                self.stake_amount
            )));
        }

        if self.commission_rate > 10000 {
            return Err(Error::InvalidData(format!(
                "Invalid commission rate: {} (maximum 10000)",
                self.commission_rate
            )));
        }

        if self.network_address.is_empty() {
            return Err(Error::InvalidData(
                "Network address cannot be empty".to_string(),
            ));
        }

        if self.p2p_address.is_empty() {
            return Err(Error::InvalidData(
                "P2P address cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}

impl fmt::Display for ValidatorMetadata {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Validator {{ address: {}, stake: {} SBTC, commission: {}% }}",
            self.silver_address,
            self.stake_amount,
            self.commission_rate as f64 / 100.0
        )
    }
}
