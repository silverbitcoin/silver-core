//! Account state and balance management
//!
//! This module manages account balances for the pure PoW blockchain.
//! All balances are immediately available (no vesting or staking).

use crate::error::{Error, Result};
use crate::SilverAddress;
use serde::{Deserialize, Serialize};

/// Account balance state
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AccountBalance {
    /// Account address
    pub address: SilverAddress,

    /// Total balance in MIST
    pub total_balance_mist: u128,

    /// Nonce for transaction ordering
    pub nonce: u64,

    /// Last update timestamp (Unix seconds)
    pub last_updated_seconds: u64,
}

impl AccountBalance {
    /// Create a new account balance
    ///
    /// # Arguments
    /// * `address` - Account address
    /// * `balance_mist` - Total balance in MIST
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// A new AccountBalance instance
    pub fn new(address: SilverAddress, balance_mist: u128, current_time_seconds: u64) -> Self {
        Self {
            address,
            total_balance_mist: balance_mist,
            nonce: 0,
            last_updated_seconds: current_time_seconds,
        }
    }

    /// Transfer balance
    ///
    /// # Arguments
    /// * `amount_mist` - Amount to transfer in MIST
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// Result of the transfer operation
    pub fn transfer(&mut self, amount_mist: u128, current_time_seconds: u64) -> Result<()> {
        if amount_mist > self.total_balance_mist {
            return Err(Error::InsufficientBalance(format!(
                "Cannot transfer {} MIST, only {} available",
                amount_mist, self.total_balance_mist
            )));
        }

        self.total_balance_mist = self.total_balance_mist.saturating_sub(amount_mist);
        self.last_updated_seconds = current_time_seconds;

        Ok(())
    }

    /// Receive balance
    ///
    /// # Arguments
    /// * `amount_mist` - Amount to receive in MIST
    /// * `current_time_seconds` - Current time (Unix timestamp)
    pub fn receive(&mut self, amount_mist: u128, current_time_seconds: u64) {
        self.total_balance_mist += amount_mist;
        self.last_updated_seconds = current_time_seconds;
    }

    /// Increment nonce for transaction ordering
    pub fn increment_nonce(&mut self) {
        self.nonce = self.nonce.saturating_add(1);
    }

    /// Check if account can transfer amount
    ///
    /// # Arguments
    /// * `amount_mist` - Amount to check in MIST
    ///
    /// # Returns
    /// true if account has sufficient balance
    pub fn can_transfer(&self, amount_mist: u128) -> bool {
        amount_mist <= self.total_balance_mist
    }
}

/// Account state store
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStore {
    /// Accounts indexed by address
    pub accounts: std::collections::BTreeMap<String, AccountBalance>,
}

impl AccountStore {
    /// Create a new account store
    pub fn new() -> Self {
        Self {
            accounts: std::collections::BTreeMap::new(),
        }
    }

    /// Get account balance
    ///
    /// # Arguments
    /// * `address` - Account address
    ///
    /// # Returns
    /// Reference to account balance, or None if not found
    pub fn get_account(&self, address: &SilverAddress) -> Option<&AccountBalance> {
        self.accounts.get(&address.to_hex())
    }

    /// Get mutable account balance
    ///
    /// # Arguments
    /// * `address` - Account address
    ///
    /// # Returns
    /// Mutable reference to account balance, or None if not found
    pub fn get_account_mut(&mut self, address: &SilverAddress) -> Option<&mut AccountBalance> {
        self.accounts.get_mut(&address.to_hex())
    }

    /// Create or get account
    ///
    /// # Arguments
    /// * `address` - Account address
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// Mutable reference to account balance
    pub fn get_or_create_account(
        &mut self,
        address: SilverAddress,
        current_time_seconds: u64,
    ) -> &mut AccountBalance {
        let addr_hex = address.to_hex();
        self.accounts
            .entry(addr_hex)
            .or_insert_with(|| AccountBalance::new(address, 0, current_time_seconds))
    }

    /// Add account
    ///
    /// # Arguments
    /// * `account` - Account balance to add
    ///
    /// # Returns
    /// Result of the operation
    pub fn add_account(&mut self, account: AccountBalance) -> Result<()> {
        let addr_hex = account.address.to_hex();
        if self.accounts.contains_key(&addr_hex) {
            return Err(Error::InvalidData("Account already exists".to_string()));
        }
        self.accounts.insert(addr_hex, account);
        Ok(())
    }

    /// Get total balance across all accounts
    ///
    /// # Returns
    /// Total balance in MIST
    pub fn get_total_balance(&self) -> u128 {
        self.accounts.values().map(|a| a.total_balance_mist).sum()
    }
}

impl Default for AccountStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SilverAddress;

    /// Generate a test address using SHA-512 hash of a seed
    fn generate_test_address(seed: &str) -> SilverAddress {
        use sha2::{Digest, Sha512};
        let mut hasher = Sha512::new();
        hasher.update(seed.as_bytes());
        let result = hasher.finalize();
        let hex = format!("{:x}", result);
        SilverAddress::from_hex(&hex).map_err(|e| format!("Failed to create test address: {}", e))?
    }

    #[test]
    fn test_account_balance_creation() {
        let addr = generate_test_address("test_account_1");
        let account = AccountBalance::new(addr.clone(), 1_000_000_000_000_000u128, 1000000u64);
        assert_eq!(account.total_balance_mist, 1_000_000_000_000_000u128);
        assert_eq!(account.nonce, 0);
        assert_eq!(account.address, addr);
    }

    #[test]
    fn test_transfer() {
        let addr = generate_test_address("test_transfer_1");
        let mut account = AccountBalance::new(addr, 1_000_000_000_000_000u128, 1000000u64);

        let result = account.transfer(100_000_000_000_000u128, 1000000u64);
        assert!(result.is_ok());
        assert_eq!(account.total_balance_mist, 900_000_000_000_000u128);
    }

    #[test]
    fn test_transfer_insufficient_balance() {
        let addr = generate_test_address("test_transfer_insufficient");
        let mut account = AccountBalance::new(addr, 100_000_000u128, 1000000u64);

        let result = account.transfer(200_000_000u128, 1000000u64);
        assert!(result.is_err());
        assert_eq!(account.total_balance_mist, 100_000_000u128);
    }

    #[test]
    fn test_receive() {
        let addr = generate_test_address("test_receive_1");
        let mut account = AccountBalance::new(addr, 1_000_000_000_000_000u128, 1000000u64);
        account.receive(500_000_000_000_000u128, 1000000u64);
        assert_eq!(account.total_balance_mist, 1_500_000_000_000_000u128);
    }

    #[test]
    fn test_nonce_increment() {
        let addr = generate_test_address("test_nonce_1");
        let mut account = AccountBalance::new(addr, 1_000_000_000_000_000u128, 1000000u64);
        assert_eq!(account.nonce, 0);
        account.increment_nonce();
        assert_eq!(account.nonce, 1);
        account.increment_nonce();
        assert_eq!(account.nonce, 2);
    }

    #[test]
    fn test_can_transfer() {
        let addr = generate_test_address("test_can_transfer");
        let account = AccountBalance::new(addr, 1_000_000u128, 1000000u64);
        assert!(account.can_transfer(500_000u128));
        assert!(account.can_transfer(1_000_000u128));
        assert!(!account.can_transfer(1_000_001u128));
    }

    #[test]
    fn test_account_store_operations() {
        let mut store = AccountStore::new();
        let addr1 = generate_test_address("store_test_1");
        let addr2 = generate_test_address("store_test_2");

        let account1 = AccountBalance::new(addr1.clone(), 500_000_000u128, 1000000u64);
        let account2 = AccountBalance::new(addr2.clone(), 300_000_000u128, 1000000u64);

        assert!(store.add_account(account1).is_ok());
        assert!(store.add_account(account2).is_ok());

        assert!(store.get_account(&addr1).is_some());
        assert!(store.get_account(&addr2).is_some());

        let total = store.get_total_balance();
        assert_eq!(total, 800_000_000u128);
    }

    #[test]
    fn test_account_store_duplicate_prevention() {
        let mut store = AccountStore::new();
        let addr = generate_test_address("duplicate_test");

        let account1 = AccountBalance::new(addr.clone(), 500_000_000u128, 1000000u64);
        let account2 = AccountBalance::new(addr.clone(), 300_000_000u128, 1000000u64);

        assert!(store.add_account(account1).is_ok());
        assert!(store.add_account(account2).is_err());
    }
}
