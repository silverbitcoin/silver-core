//! Error types for SilverBitcoin core

use thiserror::Error;

/// Core error type
#[derive(Error, Debug)]
pub enum Error {
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Invalid data error
    #[error("Invalid data: {0}")]
    InvalidData(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),

    /// Resource exhausted error
    #[error("Resource exhausted: {0}")]
    ResourceExhausted(String),

    /// Cryptographic error
    #[error("Cryptographic error: {0}")]
    Cryptographic(String),

    /// Invalid vesting amount
    #[error("Invalid vesting amount: must be greater than 0")]
    InvalidVestingAmount,

    /// Invalid vesting period
    #[error("Invalid vesting period: must be greater than 0")]
    InvalidVestingPeriod,

    /// Vesting schedule already exists
    #[error("Vesting schedule already exists for this address")]
    VestingScheduleAlreadyExists,

    /// Vesting schedule not found
    #[error("Vesting schedule not found for this address")]
    VestingScheduleNotFound,

    /// Insufficient vested balance
    #[error("Insufficient vested balance: {0}")]
    InsufficientVestedBalance(String),

    /// Locked tokens cannot be transferred
    #[error("Cannot transfer locked tokens")]
    LockedTokensCannotBeTransferred,
}

/// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
