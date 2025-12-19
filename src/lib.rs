//! # SilverBitcoin Core
//!
//! Core types, traits, and primitives for the SilverBitcoin blockchain.
//!
//! This crate provides the fundamental building blocks used throughout the
//! SilverBitcoin ecosystem, including:
//!
//! - Object model (ObjectID, SilverAddress, Object, Owner)
//! - Transaction structures (Transaction, TransactionData, Command)
//! - Consensus types (Batch, Certificate, Snapshot)
//! - Cryptographic primitives (Signature, PublicKey, Hash)
//! - Error types and result wrappers

#![warn(missing_docs, rust_2018_idioms)]
#![forbid(unsafe_code)]

/// Number of MIST per SLVR (1 SLVR = 1,000,000,000 MIST)
///
/// This provides 9 decimal places of precision for SLVR amounts.
/// Similar to Bitcoin's satoshi (1 BTC = 100,000,000 satoshis),
/// but with one extra decimal place for finer granularity.
pub const MIST_PER_SLVR: u64 = 1_000_000_000;

/// Minimum fuel price in MIST per fuel unit
///
/// This is the absolute minimum price that must be paid per fuel unit.
/// At 1000 MIST per fuel unit, this ensures spam prevention while
/// keeping fees affordable.
pub const MIN_FUEL_PRICE_MIST: u64 = 1000;

/// Address types and utilities
pub mod address;

/// Account state and balance management
pub mod account;

/// Error types and result wrappers
pub mod error;

/// Object model and identifiers
pub mod object;

/// Cryptographic signatures
pub mod signature;

/// Transaction structures and types
pub mod transaction;

/// Consensus types and structures
pub mod consensus;

/// Hash functions and types
pub mod hash;

/// Protocol definitions
pub mod protocol;

/// Token standard implementation (ERC-20 like)
pub mod token;

/// Proof-of-Work types and structures
pub mod pow;

/// Genesis block initialization
pub mod genesis;

/// Wallet and address management
pub mod wallet;

/// Privacy features (Stealth Addresses, Ring Signatures, Bulletproofs)
pub mod privacy;

/// Privacy V2 (Bulletproofs++, Batch Verification, Compression)
pub mod privacy_v2;

/// RPC API for blockchain interaction
pub mod rpc_api;

pub use account::{AccountBalance, AccountStore};
pub use address::SilverAddress;
pub use consensus::{
    BatchID, Certificate, Snapshot, SnapshotSequenceNumber, TransactionBatch, ValidatorID,
    ValidatorMetadata, ValidatorSignature,
};
pub use error::{Error, Result};
pub use hash::{Blake3Hash, SnapshotDigest, StateDigest, TransactionDigest};
pub use object::{Object, ObjectID, ObjectRef, ObjectType, Owner, SequenceNumber};
pub use protocol::{
    ApprovedUpgrade, FeatureFlags, ProposalID, ProtocolVersion, UpgradeProposal, UpgradeVote,
    VotingResults,
};
pub use signature::{PublicKey, Signature, SignatureScheme};

pub use transaction::{
    Command, Identifier, Transaction, TransactionData, TransactionExpiration, TransactionKind,
};
pub use token::{
    TokenMetadata, TokenBalance, TokenAllowance, TokenTransferEvent, TokenApprovalEvent,
    TokenMintEvent, TokenBurnEvent, TokenState,
};
pub use pow::{BlockHeader, WorkProof, MiningReward, DifficultyAdjustment};
pub use genesis::{GenesisBlock, GenesisConfig};
pub use wallet::{Wallet, WalletAddress, AddressGenerator};
pub use privacy::{
    StealthAddress, RingSignature, BulletproofPlus, PrivacyTransaction, PrivacyPool, PoolStats,
};
pub use privacy_v2::{
    BulletproofPlusPlus, BatchVerifier, BatchStats, CompressedTransaction, TransactionPoolV2,
    PoolStatsV2, PrivacyTransactionV2,
};
