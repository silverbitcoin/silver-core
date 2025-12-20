//! Object model types
//!
//! This module defines the core object model for SilverBitcoin blockchain.
//! Objects are first-class primitives with unique identifiers, versioning,
//! and flexible ownership models.

use crate::{Error, Result, SilverAddress, TransactionDigest};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// 64-byte (512-bit) quantum-resistant object identifier
///
/// Object IDs are derived from Blake3-512 hashes of object creation data,
/// providing collision resistance and quantum security.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ObjectID(pub [u8; 64]);

impl Serialize for ObjectID {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for ObjectID {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ObjectIDVisitor;

        impl<'de> serde::de::Visitor<'de> for ObjectIDVisitor {
            type Value = ObjectID;

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
                Ok(ObjectID(arr))
            }

            fn visit_seq<A>(self, mut seq: A) -> std::result::Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                let mut arr = [0u8; 64];
                for (i, elem) in arr.iter_mut().enumerate() {
                    *elem = seq
                        .next_element()?
                        .ok_or_else(|| serde::de::Error::invalid_length(i, &self))?;
                }
                Ok(ObjectID(arr))
            }
        }

        deserializer.deserialize_bytes(ObjectIDVisitor)
    }
}

impl ObjectID {
    /// Create a new ObjectID from a 64-byte array
    pub const fn new(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// Create ObjectID from a slice, returning error if wrong length
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 64 {
            return Err(Error::InvalidData(format!(
                "ObjectID must be 64 bytes, got {}",
                bytes.len()
            )));
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(bytes);
        Ok(Self(arr))
    }

    /// Get the bytes as a slice
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    /// Convert to hex string for display
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse from hex string
    pub fn from_hex(s: &str) -> Result<Self> {
        let bytes =
            hex::decode(s).map_err(|e| Error::InvalidData(format!("Invalid hex: {}", e)))?;
        Self::from_bytes(&bytes)
    }

    /// Convert to base58 string
    pub fn to_base58(&self) -> String {
        bs58::encode(&self.0).into_string()
    }

    /// Parse from base58 string
    pub fn from_base58(s: &str) -> Result<Self> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|e| Error::InvalidData(format!("Invalid base58: {}", e)))?;
        Self::from_bytes(&bytes)
    }
}

impl fmt::Debug for ObjectID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ObjectID({}...)", &self.to_hex()[..16])
    }
}

impl fmt::Display for ObjectID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base58())
    }
}

impl AsRef<[u8]> for ObjectID {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Object reference with version for tracking object state
///
/// Used to reference specific versions of objects in transactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ObjectRef {
    /// The object identifier
    pub id: ObjectID,
    /// The version number of this object
    pub version: SequenceNumber,
    /// Digest of the transaction that created this version
    pub digest: TransactionDigest,
}

impl ObjectRef {
    /// Create a new object reference
    pub const fn new(id: ObjectID, version: SequenceNumber, digest: TransactionDigest) -> Self {
        Self {
            id,
            version,
            digest,
        }
    }

    /// Check if this reference is for the initial version
    pub fn is_initial_version(&self) -> bool {
        self.version.0 == 0
    }
}

impl fmt::Display for ObjectRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@v{}", self.id, self.version.0)
    }
}

/// Monotonically increasing version number for object versioning
///
/// Each time an object is modified, its version number increments.
/// This enables tracking object history and preventing replay attacks.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SequenceNumber(pub u64);

impl SequenceNumber {
    /// Create a new sequence number
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// Get the initial sequence number (0)
    pub const fn initial() -> Self {
        Self(0)
    }

    /// Increment the sequence number
    pub fn increment(&mut self) {
        self.0 = self.0.saturating_add(1);
    }

    /// Get the next sequence number without modifying this one
    pub fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }

    /// Get the inner value
    pub const fn value(&self) -> u64 {
        self.0
    }

    /// Check if this is the initial version
    pub const fn is_initial(&self) -> bool {
        self.0 == 0
    }
}

impl fmt::Debug for SequenceNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SequenceNumber({})", self.0)
    }
}

impl fmt::Display for SequenceNumber {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for SequenceNumber {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<SequenceNumber> for u64 {
    fn from(seq: SequenceNumber) -> Self {
        seq.0
    }
}

/// Object ownership model supporting multiple ownership patterns
///
/// SilverBitcoin supports four ownership models:
/// - AddressOwner: Single address owns the object (most common)
/// - Shared: Multiple transactions can access (requires consensus)
/// - Immutable: Cannot be modified after creation
/// - ObjectOwner: Owned by another object (wrapped objects)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Owner {
    /// Single address owns this object
    /// Only transactions signed by this address can modify the object
    AddressOwner(SilverAddress),

    /// Shared object accessible by any transaction
    /// Modifications require consensus ordering
    Shared {
        /// The version when this object became shared
        initial_shared_version: SequenceNumber,
    },

    /// Immutable object that cannot be modified
    /// Can be read without consensus
    Immutable,

    /// Object is owned by another object (wrapped)
    /// Inherits parent's ownership model
    ObjectOwner(ObjectID),
}

impl Owner {
    /// Check if this is an address-owned object
    pub fn is_address_owned(&self) -> bool {
        matches!(self, Owner::AddressOwner(_))
    }

    /// Check if this is a shared object
    pub fn is_shared(&self) -> bool {
        matches!(self, Owner::Shared { .. })
    }

    /// Check if this is an immutable object
    pub fn is_immutable(&self) -> bool {
        matches!(self, Owner::Immutable)
    }

    /// Check if this is an object-owned (wrapped) object
    pub fn is_object_owned(&self) -> bool {
        matches!(self, Owner::ObjectOwner(_))
    }

    /// Get the owning address if this is address-owned
    pub fn address(&self) -> Option<&SilverAddress> {
        match self {
            Owner::AddressOwner(addr) => Some(addr),
            _ => None,
        }
    }

    /// Get the parent object ID if this is object-owned
    pub fn parent_object(&self) -> Option<&ObjectID> {
        match self {
            Owner::ObjectOwner(id) => Some(id),
            _ => None,
        }
    }

    /// Check if the given address can modify this object
    pub fn can_modify(&self, address: &SilverAddress) -> bool {
        match self {
            Owner::AddressOwner(owner) => owner == address,
            Owner::Shared { .. } => true, // Shared objects can be modified by anyone (with consensus)
            Owner::Immutable => false,
            Owner::ObjectOwner(_) => false, // Must modify through parent
        }
    }
}

impl fmt::Display for Owner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Owner::AddressOwner(addr) => write!(f, "Address({})", addr),
            Owner::Shared {
                initial_shared_version,
            } => write!(f, "Shared(v{})", initial_shared_version.0),
            Owner::Immutable => write!(f, "Immutable"),
            Owner::ObjectOwner(id) => write!(f, "Object({})", id),
        }
    }
}

/// Object data type enumeration
///
/// Defines the type of data stored in an object for type-safe operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObjectType {
    /// Quantum Move module package
    Package,
    /// Quantum Move module
    Module,
    /// Coin/token object
    Coin,
    /// Validator object
    Validator,
    /// Event object
    Event,
    /// Generic object with custom data
    Struct {
        /// Package ID containing the struct definition
        package: ObjectID,
        /// Module name
        module: String,
        /// Struct name
        name: String,
    },
}

impl fmt::Display for ObjectType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ObjectType::Package => write!(f, "Package"),
            ObjectType::Module => write!(f, "Module"),
            ObjectType::Coin => write!(f, "Coin"),
            ObjectType::Validator => write!(f, "Validator"),
            ObjectType::Event => write!(f, "Event"),
            ObjectType::Struct {
                package,
                module,
                name,
            } => write!(f, "{}::{}::{}", package, module, name),
        }
    }
}

/// Core object structure representing blockchain state
///
/// Objects are the fundamental unit of state in SilverBitcoin.
/// Each object has a unique ID, version, owner, and data payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Object {
    /// Unique 512-bit object identifier
    pub id: ObjectID,

    /// Current version number (increments on each modification)
    pub version: SequenceNumber,

    /// Ownership model for this object
    pub owner: Owner,

    /// Type of object data
    pub object_type: ObjectType,

    /// Serialized object data (format depends on object_type)
    pub data: Vec<u8>,

    /// Digest of the transaction that created this version
    pub previous_transaction: TransactionDigest,

    /// Storage rebate for deleting this object (in MIST)
    pub storage_rebate: u64,
}

impl Object {
    /// Create a new object
    pub fn new(
        id: ObjectID,
        version: SequenceNumber,
        owner: Owner,
        object_type: ObjectType,
        data: Vec<u8>,
        previous_transaction: TransactionDigest,
        storage_rebate: u64,
    ) -> Self {
        Self {
            id,
            version,
            owner,
            object_type,
            data,
            previous_transaction,
            storage_rebate,
        }
    }

    /// Get an object reference for this object
    pub fn reference(&self) -> ObjectRef {
        ObjectRef::new(self.id, self.version, self.previous_transaction)
    }

    /// Check if this object is owned by the given address
    pub fn is_owned_by(&self, address: &SilverAddress) -> bool {
        self.owner.address() == Some(address)
    }

    /// Check if this object can be modified by the given address
    pub fn can_be_modified_by(&self, address: &SilverAddress) -> bool {
        self.owner.can_modify(address)
    }

    /// Get the size of this object in bytes (for storage cost calculation)
    pub fn size_bytes(&self) -> usize {
        std::mem::size_of::<ObjectID>()
            + std::mem::size_of::<SequenceNumber>()
            + std::mem::size_of::<Owner>()
            + std::mem::size_of::<ObjectType>()
            + self.data.len()
            + std::mem::size_of::<TransactionDigest>()
            + std::mem::size_of::<u64>()
    }

    /// Validate object invariants
    pub fn validate(&self) -> Result<()> {
        // Check data is not empty for non-package objects
        if self.data.is_empty() && !matches!(self.object_type, ObjectType::Package) {
            return Err(Error::InvalidData(
                "Object data cannot be empty".to_string(),
            ));
        }

        // Validate owner consistency
        if let Owner::Shared {
                initial_shared_version,
            } = &self.owner {
            if initial_shared_version > &self.version {
                return Err(Error::InvalidData(format!(
                    "Initial shared version {} cannot be greater than current version {}",
                    initial_shared_version.0, self.version.0
                )));
            }
        }

        Ok(())
    }

    /// Create a new version of this object with updated data
    pub fn new_version(
        &self,
        new_data: Vec<u8>,
        transaction_digest: TransactionDigest,
    ) -> Result<Self> {
        if self.owner.is_immutable() {
            return Err(Error::InvalidData(
                "Cannot create new version of immutable object".to_string(),
            ));
        }

        Ok(Self {
            id: self.id,
            version: self.version.next(),
            owner: self.owner.clone(),
            object_type: self.object_type.clone(),
            data: new_data,
            previous_transaction: transaction_digest,
            storage_rebate: self.storage_rebate,
        })
    }

    /// Transfer ownership to a new address
    pub fn transfer_to(
        &self,
        new_owner: SilverAddress,
        transaction_digest: TransactionDigest,
    ) -> Result<Self> {
        if !self.owner.is_address_owned() {
            return Err(Error::InvalidData(
                "Can only transfer address-owned objects".to_string(),
            ));
        }

        Ok(Self {
            id: self.id,
            version: self.version.next(),
            owner: Owner::AddressOwner(new_owner),
            object_type: self.object_type.clone(),
            data: self.data.clone(),
            previous_transaction: transaction_digest,
            storage_rebate: self.storage_rebate,
        })
    }

    /// Make this object shared
    pub fn make_shared(&self, transaction_digest: TransactionDigest) -> Result<Self> {
        if !self.owner.is_address_owned() {
            return Err(Error::InvalidData(
                "Can only share address-owned objects".to_string(),
            ));
        }

        Ok(Self {
            id: self.id,
            version: self.version.next(),
            owner: Owner::Shared {
                initial_shared_version: self.version.next(),
            },
            object_type: self.object_type.clone(),
            data: self.data.clone(),
            previous_transaction: transaction_digest,
            storage_rebate: self.storage_rebate,
        })
    }

    /// Make this object immutable
    pub fn make_immutable(&self, transaction_digest: TransactionDigest) -> Result<Self> {
        if !self.owner.is_address_owned() {
            return Err(Error::InvalidData(
                "Can only freeze address-owned objects".to_string(),
            ));
        }

        Ok(Self {
            id: self.id,
            version: self.version.next(),
            owner: Owner::Immutable,
            object_type: self.object_type.clone(),
            data: self.data.clone(),
            previous_transaction: transaction_digest,
            storage_rebate: self.storage_rebate,
        })
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Object {{ id: {}, version: {}, owner: {}, type: {}, size: {} bytes }}",
            self.id,
            self.version,
            self.owner,
            self.object_type,
            self.size_bytes()
        )
    }
}
