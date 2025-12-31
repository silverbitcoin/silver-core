//! Transaction structures
//!
//! This module defines the transaction model for SilverBitcoin blockchain.
//! Transactions are signed operations that modify blockchain state through
//! Composite Transaction Chains (CTC).

use crate::{Error, ObjectID, ObjectRef, Result, Signature, SilverAddress, TransactionDigest};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::fmt;

/// Transaction with signatures
///
/// A complete transaction includes the transaction data and one or more signatures.
/// Multiple signatures are required for sponsored transactions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    /// The transaction data to be executed
    pub data: TransactionData,

    /// Signatures authorizing this transaction
    /// - First signature is always from the sender
    /// - Additional signatures may be from sponsor (for fuel payment)
    pub signatures: Vec<Signature>,
}

impl Transaction {
    /// Create a new transaction
    pub fn new(data: TransactionData, signatures: Vec<Signature>) -> Self {
        Self { data, signatures }
    }

    /// Compute the digest of this transaction for signing using SHA-512
    pub fn digest(&self) -> TransactionDigest {
        // Serialize transaction data using serde_json for reliable serialization
        let serialized = match serde_json::to_vec(&self.data) {
            Ok(data) => data,
            Err(_e) => {
                // Fallback: use debug format if JSON fails
                format!("{:?}", self.data).into_bytes()
            }
        };

        // Use SHA-512 for 512-bit blockchain
        let mut hasher = sha2::Sha512::new();
        hasher.update(&serialized);
        let result = hasher.finalize();

        // Convert to 64-byte array
        let mut output = [0u8; 64];
        output.copy_from_slice(&result[..]);

        TransactionDigest::new(output)
    }

    /// Get the sender address
    pub fn sender(&self) -> &SilverAddress {
        &self.data.sender
    }

    /// Get the fuel budget
    pub fn fuel_budget(&self) -> u64 {
        self.data.fuel_budget
    }

    /// Get the fuel price
    pub fn fuel_price(&self) -> u64 {
        self.data.fuel_price
    }

    /// Calculate total fuel cost
    pub fn total_fuel_cost(&self) -> u64 {
        self.data.fuel_budget.saturating_mul(self.data.fuel_price)
    }

    /// Check if this is a sponsored transaction
    pub fn is_sponsored(&self) -> bool {
        self.data.sponsor.is_some()
    }

    /// Get the sponsor address if this is a sponsored transaction
    pub fn sponsor(&self) -> Option<&SilverAddress> {
        self.data.sponsor.as_ref()
    }

    /// Validate transaction structure
    pub fn validate(&self) -> Result<()> {
        // Must have at least one signature
        if self.signatures.is_empty() {
            return Err(Error::InvalidData(
                "Transaction must have at least one signature".to_string(),
            ));
        }

        // Sponsored transactions must have exactly 2 signatures
        if self.is_sponsored() && self.signatures.len() != 2 {
            return Err(Error::InvalidData(
                "Sponsored transaction must have exactly 2 signatures".to_string(),
            ));
        }

        // Non-sponsored transactions must have exactly 1 signature
        if !self.is_sponsored() && self.signatures.len() != 1 {
            return Err(Error::InvalidData(
                "Non-sponsored transaction must have exactly 1 signature".to_string(),
            ));
        }

        // Validate transaction data
        self.data.validate()?;

        Ok(())
    }

    /// Get commands from this transaction
    pub fn commands(&self) -> Vec<&Command> {
        match &self.data.kind {
            TransactionKind::CompositeChain(commands) => commands.iter().collect(),
            TransactionKind::Genesis(_) => Vec::new(),
            TransactionKind::ConsensusCommit(_) => Vec::new(),
        }
    }

    /// Get all input objects referenced by this transaction
    pub fn input_objects(&self) -> Vec<ObjectRef> {
        let mut objects = vec![self.data.fuel_payment];

        match &self.data.kind {
            TransactionKind::CompositeChain(commands) => {
                for cmd in commands {
                    objects.extend(cmd.input_objects());
                }
            }
            TransactionKind::Genesis(_) => {}
            TransactionKind::ConsensusCommit(_) => {}
        }

        objects
    }

    /// Estimate the size of this transaction in bytes using SHA-512
    pub fn size_bytes(&self) -> usize {
        match serde_json::to_vec(self) {
            Ok(bytes) => bytes.len(),
            Err(_) => {
                // Fallback: estimate size from debug format
                format!("{:?}", self).len()
            }
        }
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Transaction {{ sender: {}, fuel: {}, commands: {} }}",
            self.sender(),
            self.fuel_budget(),
            self.data.kind.command_count()
        )
    }
}

/// Transaction data (the part that gets signed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    /// Address sending this transaction
    pub sender: SilverAddress,

    /// Object used to pay for fuel (must be owned by sender or sponsor)
    pub fuel_payment: ObjectRef,

    /// Maximum fuel units this transaction can consume
    pub fuel_budget: u64,

    /// Price per fuel unit in MIST (minimum 1000)
    pub fuel_price: u64,

    /// The kind of transaction
    pub kind: TransactionKind,

    /// Optional sponsor who pays for fuel
    pub sponsor: Option<SilverAddress>,

    /// Transaction expiration (Unix timestamp in seconds)
    pub expiration: TransactionExpiration,
}

impl TransactionData {
    /// Create new transaction data
    pub fn new(
        sender: SilverAddress,
        fuel_payment: ObjectRef,
        fuel_budget: u64,
        fuel_price: u64,
        kind: TransactionKind,
        expiration: TransactionExpiration,
    ) -> Self {
        Self {
            sender,
            fuel_payment,
            fuel_budget,
            fuel_price,
            kind,
            sponsor: None,
            expiration,
        }
    }

    /// Create sponsored transaction data
    pub fn new_sponsored(
        sender: SilverAddress,
        sponsor: SilverAddress,
        fuel_payment: ObjectRef,
        fuel_budget: u64,
        fuel_price: u64,
        kind: TransactionKind,
        expiration: TransactionExpiration,
    ) -> Self {
        Self {
            sender,
            fuel_payment,
            fuel_budget,
            fuel_price,
            kind,
            sponsor: Some(sponsor),
            expiration,
        }
    }

    /// Validate transaction data
    pub fn validate(&self) -> Result<()> {
        // Validate fuel price (minimum 1000 MIST)
        if self.fuel_price < 1000 {
            return Err(Error::InvalidData(format!(
                "Fuel price must be at least 1000 MIST, got {}",
                self.fuel_price
            )));
        }

        // Validate fuel budget (maximum 50 million)
        if self.fuel_budget > 50_000_000 {
            return Err(Error::InvalidData(format!(
                "Fuel budget cannot exceed 50,000,000, got {}",
                self.fuel_budget
            )));
        }

        // Validate fuel budget is non-zero
        if self.fuel_budget == 0 {
            return Err(Error::InvalidData(
                "Fuel budget must be greater than 0".to_string(),
            ));
        }

        // Validate transaction kind
        self.kind.validate()?;

        Ok(())
    }
}

/// Transaction expiration
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TransactionExpiration {
    /// No expiration
    None,
    /// Expires at specific Unix timestamp (seconds)
    Timestamp(u64),
    /// Expires after specific snapshot number
    Snapshot(u64),
}

impl TransactionExpiration {
    /// Check if transaction has expired
    pub fn is_expired(&self, current_time: u64, current_snapshot: u64) -> bool {
        match self {
            TransactionExpiration::None => false,
            TransactionExpiration::Timestamp(ts) => current_time > *ts,
            TransactionExpiration::Snapshot(sn) => current_snapshot > *sn,
        }
    }
}

/// Transaction kind enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionKind {
    /// Composite Transaction Chain - sequence of commands
    CompositeChain(Vec<Command>),

    /// Genesis transaction (only valid in genesis block)
    Genesis(GenesisTransaction),

    /// Consensus commit prologue (only valid from validators)
    ConsensusCommit(ConsensusCommitPrologue),
}

impl TransactionKind {
    /// Get the number of commands in this transaction
    pub fn command_count(&self) -> usize {
        match self {
            TransactionKind::CompositeChain(commands) => commands.len(),
            TransactionKind::Genesis(_) => 1,
            TransactionKind::ConsensusCommit(_) => 1,
        }
    }

    /// Validate transaction kind
    pub fn validate(&self) -> Result<()> {
        match self {
            TransactionKind::CompositeChain(commands) => {
                if commands.is_empty() {
                    return Err(Error::InvalidData(
                        "CompositeChain must have at least one command".to_string(),
                    ));
                }
                if commands.len() > 1024 {
                    return Err(Error::InvalidData(format!(
                        "CompositeChain cannot have more than 1024 commands, got {}",
                        commands.len()
                    )));
                }
                for cmd in commands {
                    cmd.validate()?;
                }
                Ok(())
            }
            TransactionKind::Genesis(_) => Ok(()),
            TransactionKind::ConsensusCommit(_) => Ok(()),
        }
    }
}

/// Genesis transaction for network initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisTransaction {
    /// Initial objects to create
    pub objects: Vec<GenesisObject>,
}

/// Object created in genesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisObject {
    /// Object ID
    pub id: ObjectID,
    /// Owner address
    pub owner: SilverAddress,
    /// Object data
    pub data: Vec<u8>,
}

/// Consensus commit prologue (validator-only transaction)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusCommitPrologue {
    /// Snapshot sequence number
    pub snapshot: u64,
    /// Timestamp
    pub timestamp: u64,
}

/// Transaction command for Composite Transaction Chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Command {
    /// Transfer objects to a recipient
    TransferObjects {
        /// Objects to transfer
        objects: Vec<ObjectRef>,
        /// Recipient address
        recipient: SilverAddress,
    },

    /// Split coins into multiple coins
    SplitCoins {
        /// Coin to split
        coin: ObjectRef,
        /// Amounts for new coins
        amounts: Vec<u64>,
    },

    /// Merge multiple coins into one
    MergeCoins {
        /// Primary coin (receives merged value)
        primary: ObjectRef,
        /// Coins to merge into primary
        coins: Vec<ObjectRef>,
    },

    /// Publish Quantum Move modules
    Publish {
        /// Compiled module bytecode
        modules: Vec<Vec<u8>>,
    },

    /// Call a Quantum Move function
    Call {
        /// Package containing the module
        package: ObjectID,
        /// Module name
        module: Identifier,
        /// Function name
        function: Identifier,
        /// Type arguments for generics
        type_arguments: Vec<TypeTag>,
        /// Function arguments
        arguments: Vec<CallArg>,
    },

    /// Make a Move vector
    MakeMoveVec {
        /// Element type (None for empty vector)
        element_type: Option<TypeTag>,
        /// Elements
        elements: Vec<CallArg>,
    },

    /// Delete an object
    DeleteObject {
        /// Object to delete
        object: ObjectRef,
    },

    /// Make an object shared
    ShareObject {
        /// Object to share
        object: ObjectRef,
    },

    /// Make an object immutable
    FreezeObject {
        /// Object to freeze
        object: ObjectRef,
    },
}

impl Command {
    /// Get input objects referenced by this command
    pub fn input_objects(&self) -> Vec<ObjectRef> {
        match self {
            Command::TransferObjects { objects, .. } => objects.clone(),
            Command::SplitCoins { coin, .. } => vec![*coin],
            Command::MergeCoins { primary, coins } => {
                let mut objs = vec![*primary];
                objs.extend(coins);
                objs
            }
            Command::Publish { .. } => vec![],
            Command::Call { arguments, .. } => arguments
                .iter()
                .filter_map(|arg| match arg {
                    CallArg::Object(obj_ref) => Some(*obj_ref),
                    _ => None,
                })
                .collect(),
            Command::MakeMoveVec { elements, .. } => elements
                .iter()
                .filter_map(|arg| match arg {
                    CallArg::Object(obj_ref) => Some(*obj_ref),
                    _ => None,
                })
                .collect(),
            Command::DeleteObject { object } => vec![*object],
            Command::ShareObject { object } => vec![*object],
            Command::FreezeObject { object } => vec![*object],
        }
    }

    /// Validate command structure
    pub fn validate(&self) -> Result<()> {
        match self {
            Command::TransferObjects { objects, .. } => {
                if objects.is_empty() {
                    return Err(Error::InvalidData(
                        "TransferObjects must have at least one object".to_string(),
                    ));
                }
                Ok(())
            }
            Command::SplitCoins { amounts, .. } => {
                if amounts.is_empty() {
                    return Err(Error::InvalidData(
                        "SplitCoins must have at least one amount".to_string(),
                    ));
                }
                if amounts.contains(&0) {
                    return Err(Error::InvalidData(
                        "SplitCoins amounts must be greater than 0".to_string(),
                    ));
                }
                Ok(())
            }
            Command::MergeCoins { coins, .. } => {
                if coins.is_empty() {
                    return Err(Error::InvalidData(
                        "MergeCoins must have at least one coin to merge".to_string(),
                    ));
                }
                Ok(())
            }
            Command::Publish { modules } => {
                if modules.is_empty() {
                    return Err(Error::InvalidData(
                        "Publish must have at least one module".to_string(),
                    ));
                }
                Ok(())
            }
            Command::Call {
                module, function, ..
            } => {
                if module.0.is_empty() {
                    return Err(Error::InvalidData(
                        "Module name cannot be empty".to_string(),
                    ));
                }
                if function.0.is_empty() {
                    return Err(Error::InvalidData(
                        "Function name cannot be empty".to_string(),
                    ));
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

/// Identifier for modules and functions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Identifier(pub String);

impl Identifier {
    /// Create a new identifier
    pub fn new(s: impl Into<String>) -> Result<Self> {
        let s = s.into();
        if s.is_empty() {
            return Err(Error::InvalidData("Identifier cannot be empty".to_string()));
        }
        if !s.chars().all(|c| c.is_alphanumeric() || c == '_') {
            return Err(Error::InvalidData(
                "Identifier can only contain alphanumeric characters and underscores".to_string(),
            ));
        }
        Ok(Self(s))
    }

    /// Get the string value
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Type tag for Quantum Move types used in smart contract function calls
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TypeTag {
    /// Boolean type
    Bool,
    /// Unsigned 8-bit integer
    U8,
    /// Unsigned 64-bit integer
    U64,
    /// Unsigned 128-bit integer
    U128,
    /// SilverBitcoin address type (512-bit)
    Address,
    /// Vector type with element type
    Vector(Box<TypeTag>),
    /// Struct type with full qualification
    Struct {
        /// Package (module bundle) ID
        package: ObjectID,
        /// Module name within the package
        module: Identifier,
        /// Struct name within the module
        name: Identifier,
        /// Type parameters for generic structs
        type_params: Vec<TypeTag>,
    },
}

impl fmt::Display for TypeTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TypeTag::Bool => write!(f, "bool"),
            TypeTag::U8 => write!(f, "u8"),
            TypeTag::U64 => write!(f, "u64"),
            TypeTag::U128 => write!(f, "u128"),
            TypeTag::Address => write!(f, "address"),
            TypeTag::Vector(inner) => write!(f, "vector<{}>", inner),
            TypeTag::Struct {
                package,
                module,
                name,
                type_params,
            } => {
                write!(f, "{}::{}::{}", package, module, name)?;
                if !type_params.is_empty() {
                    write!(f, "<")?;
                    for (i, param) in type_params.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", param)?;
                    }
                    write!(f, ">")?;
                }
                Ok(())
            }
        }
    }
}

/// Call argument for function calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CallArg {
    /// Pure value (serialized)
    Pure(Vec<u8>),
    /// Object reference
    Object(ObjectRef),
    /// Result from previous command
    Result(u16),
    /// Nested result from previous command
    NestedResult(u16, u16),
}
