//! Cryptographic signature types

use serde::{Deserialize, Serialize};

/// Signature scheme enumeration for quantum-resistant cryptography
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureScheme {
    /// Secp256k1 (Bitcoin/Ethereum standard) - 256-bit elliptic curve signature
    Secp256k1,
    /// SPHINCS+ post-quantum hash-based signature scheme (NIST standard)
    SphincsPlus,
    /// Dilithium3 post-quantum lattice-based signature scheme (NIST standard)
    Dilithium3,
    /// Secp512r1 (NIST P-521) classical 512-bit elliptic curve signature
    Secp512r1,
    /// Hybrid mode combining classical and post-quantum signatures
    Hybrid,
}

/// Public key wrapper containing the signature scheme and key bytes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublicKey {
    /// The signature scheme used for this public key
    pub scheme: SignatureScheme,
    /// Raw public key bytes (size varies by scheme)
    pub bytes: Vec<u8>,
}

/// Signature wrapper containing the signature scheme and signature bytes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    /// The signature scheme used to generate this signature
    pub scheme: SignatureScheme,
    /// Raw signature bytes (size varies by scheme: 64 bytes for Secp256k1, ~49KB for SPHINCS+, ~3.3KB for Dilithium3, 132 bytes for Secp512r1)
    pub bytes: Vec<u8>,
}

impl Signature {
    /// Get signature bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl PublicKey {
    /// Get public key bytes
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl AsRef<[u8]> for PublicKey {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}
