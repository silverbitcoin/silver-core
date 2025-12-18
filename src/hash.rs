//! Hash types (512-bit Blake3)
//!
//! All hashes in SilverBitcoin use Blake3-512 for quantum resistance.

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

/// Transaction digest (512-bit Blake3 hash)
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TransactionDigest(pub [u8; 64]);

impl_serde_64!(TransactionDigest);

impl TransactionDigest {
    /// Create a new transaction digest
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
}

impl fmt::Debug for TransactionDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "TransactionDigest({}...)", &self.to_hex()[..16])
    }
}

impl fmt::Display for TransactionDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_hex()[..16])
    }
}

impl AsRef<[u8]> for TransactionDigest {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

/// Snapshot digest (512-bit Blake3 hash)
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SnapshotDigest(pub [u8; 64]);

impl_serde_64!(SnapshotDigest);

impl SnapshotDigest {
    /// Create a new snapshot digest
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
}

impl fmt::Debug for SnapshotDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SnapshotDigest({}...)", &self.to_hex()[..16])
    }
}

impl fmt::Display for SnapshotDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_hex()[..16])
    }
}

/// State digest (512-bit Blake3 hash)
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct StateDigest(pub [u8; 64]);

impl_serde_64!(StateDigest);

impl StateDigest {
    /// Create a new state digest
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
}

impl fmt::Debug for StateDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "StateDigest({}...)", &self.to_hex()[..16])
    }
}

impl fmt::Display for StateDigest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_hex()[..16])
    }
}

/// Generic Blake3-512 hash
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Blake3Hash(pub [u8; 64]);

impl_serde_64!(Blake3Hash);

impl Blake3Hash {
    /// Create a new Blake3 hash
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

    /// Compute Blake3-512 hash of data
    pub fn hash(data: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(data);
        let mut output = [0u8; 64];
        hasher.finalize_xof().fill(&mut output);
        Self(output)
    }
}

impl fmt::Debug for Blake3Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Blake3Hash({}...)", &self.to_hex()[..16])
    }
}

impl fmt::Display for Blake3Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_hex()[..16])
    }
}
