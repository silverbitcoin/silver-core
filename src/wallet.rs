//! Wallet and address management for SilverBitcoin
//! Handles key generation, address creation, and transaction signing

use ed25519_dalek::SigningKey;
use rand::Rng;
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::HashMap;
use tracing::info;

/// Wallet for managing addresses and keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// Wallet name
    pub name: String,
    /// Addresses in this wallet
    pub addresses: HashMap<String, WalletAddress>,
    /// Default address
    pub default_address: Option<String>,
}

/// Address with associated keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAddress {
    /// Address string
    pub address: String,
    /// Public key (hex)
    pub public_key: String,
    /// Private key (hex) - NEVER expose in production
    pub private_key: String,
    /// Address label
    pub label: String,
    /// Creation timestamp
    pub created_at: u64,
    /// Balance
    pub balance: u128,
}

impl Wallet {
    /// Create a new wallet
    pub fn new(name: String) -> Self {
        info!("Creating new wallet: {}", name);

        Self {
            name,
            addresses: HashMap::new(),
            default_address: None,
        }
    }

    /// Generate a new address in this wallet
    pub fn generate_address(&mut self, label: Option<String>) -> Result<String, Box<dyn std::error::Error>> {
        // Generate random seed
        let mut rng = rand::thread_rng();
        let seed: [u8; 32] = rng.gen();

        // Create signing key from seed
        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();

        // Create address from public key using SHA-512
        let public_key_bytes = verifying_key.as_bytes();
        let address_hash = sha2::Sha512::digest(public_key_bytes);
        // Take first 32 bytes of SHA-512 hash for address
        let address_bytes = &address_hash[..32];
        let address = format!("SLVR{}", bs58::encode(address_bytes).into_string());

        let wallet_address = WalletAddress {
            address: address.clone(),
            public_key: hex::encode(public_key_bytes),
            private_key: hex::encode(seed),
            label: label.unwrap_or_else(|| format!("Address {}", self.addresses.len() + 1)),
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            balance: 0,
        };

        self.addresses.insert(address.clone(), wallet_address);

        // Set as default if first address
        if self.default_address.is_none() {
            self.default_address = Some(address.clone());
        }

        info!("Generated new address: {}", address);

        Ok(address)
    }

    /// Get address by string
    pub fn get_address(&self, address: &str) -> Option<&WalletAddress> {
        self.addresses.get(address)
    }

    /// Get mutable address
    pub fn get_address_mut(&mut self, address: &str) -> Option<&mut WalletAddress> {
        self.addresses.get_mut(address)
    }

    /// Get default address
    pub fn get_default_address(&self) -> Option<&WalletAddress> {
        self.default_address
            .as_ref()
            .and_then(|addr| self.addresses.get(addr))
    }

    /// Set default address
    pub fn set_default_address(&mut self, address: String) -> Result<(), String> {
        if self.addresses.contains_key(&address) {
            self.default_address = Some(address);
            Ok(())
        } else {
            Err(format!("Address not found: {}", address))
        }
    }

    /// Get all addresses
    pub fn list_addresses(&self) -> Vec<&WalletAddress> {
        self.addresses.values().collect()
    }

    /// Get total balance
    pub fn get_total_balance(&self) -> u128 {
        self.addresses.values().map(|a| a.balance).sum()
    }

    /// Update address balance
    pub fn update_balance(&mut self, address: &str, balance: u128) -> Result<(), String> {
        if let Some(wallet_addr) = self.addresses.get_mut(address) {
            wallet_addr.balance = balance;
            Ok(())
        } else {
            Err(format!("Address not found: {}", address))
        }
    }

    /// Add balance to address
    pub fn add_balance(&mut self, address: &str, amount: u128) -> Result<(), String> {
        if let Some(wallet_addr) = self.addresses.get_mut(address) {
            wallet_addr.balance = wallet_addr.balance.saturating_add(amount);
            Ok(())
        } else {
            Err(format!("Address not found: {}", address))
        }
    }

    /// Subtract balance from address
    pub fn subtract_balance(&mut self, address: &str, amount: u128) -> Result<(), String> {
        if let Some(wallet_addr) = self.addresses.get_mut(address) {
            if wallet_addr.balance >= amount {
                wallet_addr.balance -= amount;
                Ok(())
            } else {
                Err(format!("Insufficient balance in address: {}", address))
            }
        } else {
            Err(format!("Address not found: {}", address))
        }
    }

    /// Export wallet to JSON (WARNING: includes private keys!)
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Import wallet from JSON
    pub fn import_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// Address generator utility
pub struct AddressGenerator;

impl AddressGenerator {
    /// Generate a new random address
    pub fn generate() -> Result<(String, String, String), Box<dyn std::error::Error>> {
        let mut rng = rand::thread_rng();
        let seed: [u8; 32] = rng.gen();

        let signing_key = SigningKey::from_bytes(&seed);
        let verifying_key = signing_key.verifying_key();

        let public_key_bytes = verifying_key.as_bytes();
        let address_hash = sha2::Sha512::digest(public_key_bytes);
        // Take first 32 bytes of SHA-512 hash for address
        let address_bytes = &address_hash[..32];
        let address = format!("SLVR{}", bs58::encode(address_bytes).into_string());

        Ok((
            address,
            hex::encode(public_key_bytes),
            hex::encode(seed),
        ))
    }

    /// Generate multiple addresses
    pub fn generate_batch(count: usize) -> Result<Vec<(String, String, String)>, Box<dyn std::error::Error>> {
        let mut addresses = Vec::new();
        for _ in 0..count {
            addresses.push(Self::generate()?);
        }
        Ok(addresses)
    }

    /// Validate address format
    pub fn validate_address(address: &str) -> bool {
        if !address.starts_with("SLVR") {
            return false;
        }

        if address.len() < 10 {
            return false;
        }

        // Try to decode from base58
        bs58::decode(&address[4..]).into_vec().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new("test_wallet".to_string());
        assert_eq!(wallet.name, "test_wallet");
        assert_eq!(wallet.addresses.len(), 0);
    }

    #[test]
    fn test_generate_address() {
        let mut wallet = Wallet::new("test_wallet".to_string());
        let address = wallet.generate_address(None).unwrap();

        assert!(address.starts_with("SLVR"));
        assert_eq!(wallet.addresses.len(), 1);
        assert_eq!(wallet.default_address, Some(address.clone()));
    }

    #[test]
    fn test_generate_multiple_addresses() {
        let mut wallet = Wallet::new("test_wallet".to_string());

        for i in 0..5 {
            let address = wallet.generate_address(Some(format!("Address {}", i))).unwrap();
            assert!(address.starts_with("SLVR"));
        }

        assert_eq!(wallet.addresses.len(), 5);
    }

    #[test]
    fn test_get_address() {
        let mut wallet = Wallet::new("test_wallet".to_string());
        let address = wallet.generate_address(None).unwrap();

        let wallet_addr = wallet.get_address(&address);
        assert!(wallet_addr.is_some());
    }

    #[test]
    fn test_balance_operations() {
        let mut wallet = Wallet::new("test_wallet".to_string());
        let address = wallet.generate_address(None).unwrap();

        wallet.update_balance(&address, 1000).unwrap();
        assert_eq!(wallet.get_address(&address).unwrap().balance, 1000);

        wallet.add_balance(&address, 500).unwrap();
        assert_eq!(wallet.get_address(&address).unwrap().balance, 1500);

        wallet.subtract_balance(&address, 300).unwrap();
        assert_eq!(wallet.get_address(&address).unwrap().balance, 1200);
    }

    #[test]
    fn test_address_generator() {
        let (address, public_key, private_key) = AddressGenerator::generate().unwrap();

        assert!(address.starts_with("SLVR"));
        assert_eq!(public_key.len(), 64); // 32 bytes = 64 hex chars
        assert_eq!(private_key.len(), 64);
    }

    #[test]
    fn test_validate_address() {
        let (address, _, _) = AddressGenerator::generate().unwrap();
        assert!(AddressGenerator::validate_address(&address));
        assert!(!AddressGenerator::validate_address("INVALID"));
    }

    #[test]
    fn test_wallet_export_import() {
        let mut wallet = Wallet::new("test_wallet".to_string());
        wallet.generate_address(None).unwrap();

        let json = wallet.export_json().unwrap();
        let imported = Wallet::import_json(&json).unwrap();

        assert_eq!(wallet.name, imported.name);
        assert_eq!(wallet.addresses.len(), imported.addresses.len());
    }
}
