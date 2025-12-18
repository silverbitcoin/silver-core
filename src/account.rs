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
    pub fn new(
        address: SilverAddress,
        balance_mist: u128,
        current_time_seconds: u64,
    ) -> Self {
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

    #[test]
    fn test_account_balance_creation() {
        let addr = SilverAddress::from_hex(
            "46c56dbc9b8169bfc189f9c13b61e46b1fea3ffd27a29c33224d03ef9ebfcb13f0b0391c2d3763479fe5608ba1848080d7e8debe9a8b9f0cc1a72e33af627d22"
        ).unwrap();

        let account = AccountBalance::new(addr, 1_000_000_000_000_000u128, 1000000u64);
        assert_eq!(account.total_balance_mist, 1_000_000_000_000_000u128);
        assert_eq!(account.nonce, 0);
    }

    #[test]
    fn test_transfer() {
        let addr = SilverAddress::from_hex(
            "46c56dbc9b8169bfc189f9c13b61e46b1fea3ffd27a29c33224d03ef9ebfcb13f0b0391c2d3763479fe5608ba1848080d7e8debe9a8b9f0cc1a72e33af627d22"
        ).unwrap();

        let mut account = AccountBalance::new(addr, 1_000_000_000_000_000u128, 1000000u64);

        let result = account.transfer(100_000_000_000_000u128, 1000000u64);
        assert!(result.is_ok());
        assert_eq!(account.total_balance_mist, 900_000_000_000_000u128);
    }

    #[test]
    fn test_receive() {
        let addr = SilverAddress::from_hex(
            "46c56dbc9b8169bfc189f9c13b61e46b1fea3ffd27a29c33224d03ef9ebfcb13f0b0391c2d3763479fe5608ba1848080d7e8debe9a8b9f0cc1a72e33af627d22"
        ).unwrap();

        let mut account = AccountBalance::new(addr, 1_000_000_000_000_000u128, 1000000u64);
        account.receive(500_000_000_000_000u128, 1000000u64);
        assert_eq!(account.total_balance_mist, 1_500_000_000_000_000u128);
    }

    #[test]
    fn test_nonce_increment() {
        let addr = SilverAddress::from_hex(
            "46c56dbc9b8169bfc189f9c13b61e46b1fea3ffd27a29c33224d03ef9ebfcb13f0b0391c2d3763479fe5608ba1848080d7e8debe9a8b9f0cc1a72e33af627d22"
        ).unwrap();

        let mut account = AccountBalance::new(addr, 1_000_000_000_000_000u128, 1000000u64);
        assert_eq!(account.nonce, 0);
        account.increment_nonce();
        assert_eq!(account.nonce, 1);
    }
}
