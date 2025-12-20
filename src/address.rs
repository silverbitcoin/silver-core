//! SilverBitcoin address types (512-bit quantum-resistant addresses)
//!
//! Addresses are derived from public keys using Blake3-512 hashing,
//! providing quantum resistance and collision resistance.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;

/// 512-bit quantum-resistant address derived from public keys using Blake3-512
///
/// SilverBitcoin uses 512-bit addresses to provide quantum resistance.
/// Addresses are derived by hashing public keys with Blake3-512.
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SilverAddress(pub [u8; 64]);

impl Serialize for SilverAddress {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for SilverAddress {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SilverAddressVisitor;

        impl<'de> serde::de::Visitor<'de> for SilverAddressVisitor {
            type Value = SilverAddress;

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
                Ok(SilverAddress(arr))
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
                Ok(SilverAddress(arr))
            }
        }

        deserializer.deserialize_bytes(SilverAddressVisitor)
    }
}

impl SilverAddress {
    /// Create a new address from 64 bytes
    pub const fn new(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    /// Create address from a slice, returning error if wrong length
    pub fn from_bytes(bytes: &[u8]) -> crate::Result<Self> {
        if bytes.len() != 64 {
            return Err(crate::Error::InvalidData(format!(
                "SilverAddress must be 64 bytes, got {}",
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

    /// Convert to Vec<u8>
    pub fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }

    /// Convert to hex string
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// Parse from hex string
    pub fn from_hex(s: &str) -> crate::Result<Self> {
        let bytes =
            hex::decode(s).map_err(|e| crate::Error::InvalidData(format!("Invalid hex: {}", e)))?;
        Self::from_bytes(&bytes)
    }

    /// Convert to base58 string (more compact representation)
    pub fn to_base58(&self) -> String {
        bs58::encode(&self.0).into_string()
    }

    /// Parse from base58 string
    pub fn from_base58(s: &str) -> crate::Result<Self> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|e| crate::Error::InvalidData(format!("Invalid base58: {}", e)))?;
        Self::from_bytes(&bytes)
    }

    /// Derive address from public key using Blake3-512
    pub fn from_public_key(public_key: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new();
        hasher.update(public_key);
        let mut output = [0u8; 64];
        hasher.finalize_xof().fill(&mut output);
        Self(output)
    }
}

impl fmt::Debug for SilverAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "SilverAddress({}...)", &self.to_hex()[..16])
    }
}

impl fmt::Display for SilverAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_base58())
    }
}

impl AsRef<[u8]> for SilverAddress {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}
