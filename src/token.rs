//! Token standard implementation
//!
//! Implements a complete ERC-20-like token standard with full support for:
//! - Token creation and initialization
//! - Transfer operations
//! - Approval and allowance management
//! - Minting and burning
//! - Event emission

use crate::{Error, Result, SilverAddress, TransactionDigest};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenMetadata {
    /// Token name (e.g., "SilverBitcoin Token")
    pub name: String,
    /// Token symbol (e.g., "SBTK")
    pub symbol: String,
    /// Number of decimal places
    pub decimals: u8,
    /// Total supply in smallest units
    pub total_supply: u128,
    /// Token creator/owner
    pub creator: SilverAddress,
    /// Creation timestamp
    pub created_at: u64,
    /// Token contract address
    pub contract_address: SilverAddress,
}

/// Token balance entry
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenBalance {
    /// Account address
    pub account: SilverAddress,
    /// Balance in smallest units
    pub amount: u128,
}

/// Token allowance entry
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenAllowance {
    /// Token owner
    pub owner: SilverAddress,
    /// Approved spender
    pub spender: SilverAddress,
    /// Allowed amount in smallest units
    pub amount: u128,
}

/// Token transfer event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenTransferEvent {
    /// Token contract address
    pub token: SilverAddress,
    /// Sender address
    pub from: SilverAddress,
    /// Recipient address
    pub to: SilverAddress,
    /// Amount transferred in smallest units
    pub amount: u128,
    /// Transaction digest
    pub tx_digest: TransactionDigest,
    /// Block number
    pub block_number: u64,
    /// Event index in block
    pub event_index: u32,
}

/// Token approval event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenApprovalEvent {
    /// Token contract address
    pub token: SilverAddress,
    /// Token owner
    pub owner: SilverAddress,
    /// Approved spender
    pub spender: SilverAddress,
    /// Approved amount in smallest units
    pub amount: u128,
    /// Transaction digest
    pub tx_digest: TransactionDigest,
    /// Block number
    pub block_number: u64,
    /// Event index in block
    pub event_index: u32,
}

/// Token mint event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenMintEvent {
    /// Token contract address
    pub token: SilverAddress,
    /// Recipient address
    pub to: SilverAddress,
    /// Amount minted in smallest units
    pub amount: u128,
    /// Transaction digest
    pub tx_digest: TransactionDigest,
    /// Block number
    pub block_number: u64,
    /// Event index in block
    pub event_index: u32,
}

/// Token burn event
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TokenBurnEvent {
    /// Token contract address
    pub token: SilverAddress,
    /// Account that burned tokens
    pub from: SilverAddress,
    /// Amount burned in smallest units
    pub amount: u128,
    /// Transaction digest
    pub tx_digest: TransactionDigest,
    /// Block number
    pub block_number: u64,
    /// Event index in block
    pub event_index: u32,
}

/// Token state - complete token contract state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenState {
    /// Token metadata
    pub metadata: TokenMetadata,
    /// All balances: address -> amount
    pub balances: HashMap<SilverAddress, u128>,
    /// All allowances: (owner, spender) -> amount
    pub allowances: HashMap<(SilverAddress, SilverAddress), u128>,
    /// Minting enabled flag
    pub minting_enabled: bool,
    /// Burning enabled flag
    pub burning_enabled: bool,
    /// Paused flag
    pub paused: bool,
    /// Owner/admin address
    pub owner: SilverAddress,
}

impl TokenState {
    /// Create a new token state
    pub fn new(
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: u128,
        creator: SilverAddress,
        contract_address: SilverAddress,
        created_at: u64,
    ) -> Self {
        let mut balances = HashMap::new();
        balances.insert(creator, initial_supply);

        Self {
            metadata: TokenMetadata {
                name,
                symbol,
                decimals,
                total_supply: initial_supply,
                creator,
                created_at,
                contract_address,
            },
            balances,
            allowances: HashMap::new(),
            minting_enabled: true,
            burning_enabled: true,
            paused: false,
            owner: creator,
        }
    }

    /// Get balance of an account
    pub fn balance_of(&self, account: &SilverAddress) -> u128 {
        self.balances.get(account).copied().unwrap_or(0)
    }

    /// Get allowance from owner to spender
    pub fn allowance(&self, owner: &SilverAddress, spender: &SilverAddress) -> u128 {
        self.allowances
            .get(&(*owner, *spender))
            .copied()
            .unwrap_or(0)
    }

    /// Transfer tokens from one account to another
    pub fn transfer(
        &mut self,
        from: &SilverAddress,
        to: &SilverAddress,
        amount: u128,
    ) -> Result<()> {
        if self.paused {
            return Err(Error::InvalidData("Token transfers are paused".to_string()));
        }

        if amount == 0 {
            return Err(Error::InvalidData("Transfer amount must be greater than 0".to_string()));
        }

        let from_balance = self.balance_of(from);
        if from_balance < amount {
            return Err(Error::InvalidData(format!(
                "Insufficient balance: {} < {}",
                from_balance, amount
            )));
        }

        // Update balances
        self.balances.insert(*from, from_balance - amount);
        let to_balance = self.balance_of(to);
        self.balances.insert(*to, to_balance + amount);

        Ok(())
    }

    /// Transfer tokens on behalf of owner (using allowance)
    pub fn transfer_from(
        &mut self,
        owner: &SilverAddress,
        spender: &SilverAddress,
        to: &SilverAddress,
        amount: u128,
    ) -> Result<()> {
        if self.paused {
            return Err(Error::InvalidData("Token transfers are paused".to_string()));
        }

        if amount == 0 {
            return Err(Error::InvalidData("Transfer amount must be greater than 0".to_string()));
        }

        // Check allowance
        let current_allowance = self.allowance(owner, spender);
        if current_allowance < amount {
            return Err(Error::InvalidData(format!(
                "Insufficient allowance: {} < {}",
                current_allowance, amount
            )));
        }

        // Check owner balance
        let owner_balance = self.balance_of(owner);
        if owner_balance < amount {
            return Err(Error::InvalidData(format!(
                "Insufficient balance: {} < {}",
                owner_balance, amount
            )));
        }

        // Update allowance
        self.allowances
            .insert((*owner, *spender), current_allowance - amount);

        // Perform transfer
        self.transfer(owner, to, amount)?;

        Ok(())
    }

    /// Approve spender to spend tokens on behalf of owner
    pub fn approve(
        &mut self,
        owner: &SilverAddress,
        spender: &SilverAddress,
        amount: u128,
    ) -> Result<()> {
        if self.paused {
            return Err(Error::InvalidData("Token approvals are paused".to_string()));
        }

        self.allowances.insert((*owner, *spender), amount);
        Ok(())
    }

    /// Mint new tokens
    pub fn mint(&mut self, to: &SilverAddress, amount: u128) -> Result<()> {
        if !self.minting_enabled {
            return Err(Error::InvalidData("Minting is disabled".to_string()));
        }

        if self.paused {
            return Err(Error::InvalidData("Token minting is paused".to_string()));
        }

        if amount == 0 {
            return Err(Error::InvalidData("Mint amount must be greater than 0".to_string()));
        }

        // Check for overflow
        let new_total = self
            .metadata
            .total_supply
            .checked_add(amount)
            .ok_or_else(|| Error::InvalidData("Total supply overflow".to_string()))?;

        let to_balance = self.balance_of(to);
        let new_balance = to_balance
            .checked_add(amount)
            .ok_or_else(|| Error::InvalidData("Balance overflow".to_string()))?;

        self.metadata.total_supply = new_total;
        self.balances.insert(*to, new_balance);

        Ok(())
    }

    /// Burn tokens
    pub fn burn(&mut self, from: &SilverAddress, amount: u128) -> Result<()> {
        if !self.burning_enabled {
            return Err(Error::InvalidData("Burning is disabled".to_string()));
        }

        if self.paused {
            return Err(Error::InvalidData("Token burning is paused".to_string()));
        }

        if amount == 0 {
            return Err(Error::InvalidData("Burn amount must be greater than 0".to_string()));
        }

        let from_balance = self.balance_of(from);
        if from_balance < amount {
            return Err(Error::InvalidData(format!(
                "Insufficient balance to burn: {} < {}",
                from_balance, amount
            )));
        }

        self.metadata.total_supply -= amount;
        self.balances.insert(*from, from_balance - amount);

        Ok(())
    }

    /// Pause token transfers
    pub fn pause(&mut self) -> Result<()> {
        self.paused = true;
        Ok(())
    }

    /// Resume token transfers
    pub fn unpause(&mut self) -> Result<()> {
        self.paused = false;
        Ok(())
    }

    /// Enable minting
    pub fn enable_minting(&mut self) -> Result<()> {
        self.minting_enabled = true;
        Ok(())
    }

    /// Disable minting
    pub fn disable_minting(&mut self) -> Result<()> {
        self.minting_enabled = false;
        Ok(())
    }

    /// Enable burning
    pub fn enable_burning(&mut self) -> Result<()> {
        self.burning_enabled = true;
        Ok(())
    }

    /// Disable burning
    pub fn disable_burning(&mut self) -> Result<()> {
        self.burning_enabled = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_creation() {
        let creator = SilverAddress::from_bytes(&[1u8; 64]).unwrap();
        let contract = SilverAddress::from_bytes(&[2u8; 64]).unwrap();
        let state = TokenState::new(
            "Test Token".to_string(),
            "TEST".to_string(),
            18,
            1_000_000_000_000_000_000_000u128,
            creator,
            contract,
            0,
        );

        assert_eq!(state.metadata.name, "Test Token");
        assert_eq!(state.metadata.symbol, "TEST");
        assert_eq!(state.balance_of(&creator), 1_000_000_000_000_000_000_000u128);
    }

    #[test]
    fn test_transfer() {
        let creator = SilverAddress::from_bytes(&[1u8; 64]).unwrap();
        let recipient = SilverAddress::from_bytes(&[2u8; 64]).unwrap();
        let contract = SilverAddress::from_bytes(&[3u8; 64]).unwrap();
        let mut state = TokenState::new(
            "Test Token".to_string(),
            "TEST".to_string(),
            18,
            1_000_000_000_000_000_000_000u128,
            creator,
            contract,
            0,
        );

        state.transfer(&creator, &recipient, 100).unwrap();
        assert_eq!(state.balance_of(&creator), 1_000_000_000_000_000_000_000u128 - 100);
        assert_eq!(state.balance_of(&recipient), 100);
    }

    #[test]
    fn test_approve_and_transfer_from() {
        let owner = SilverAddress::from_bytes(&[1u8; 64]).unwrap();
        let spender = SilverAddress::from_bytes(&[2u8; 64]).unwrap();
        let recipient = SilverAddress::from_bytes(&[3u8; 64]).unwrap();
        let contract = SilverAddress::from_bytes(&[4u8; 64]).unwrap();
        let mut state = TokenState::new(
            "Test Token".to_string(),
            "TEST".to_string(),
            18,
            1_000_000_000_000_000_000_000u128,
            owner,
            contract,
            0,
        );

        state.approve(&owner, &spender, 500).unwrap();
        assert_eq!(state.allowance(&owner, &spender), 500);

        state
            .transfer_from(&owner, &spender, &recipient, 300)
            .unwrap();
        assert_eq!(state.balance_of(&recipient), 300);
        assert_eq!(state.allowance(&owner, &spender), 200);
    }

    #[test]
    fn test_mint() {
        let creator = SilverAddress::from_bytes(&[1u8; 64]).unwrap();
        let recipient = SilverAddress::from_bytes(&[2u8; 64]).unwrap();
        let contract = SilverAddress::from_bytes(&[3u8; 64]).unwrap();
        let mut state = TokenState::new(
            "Test Token".to_string(),
            "TEST".to_string(),
            18,
            1_000_000_000_000_000_000_000u128,
            creator,
            contract,
            0,
        );

        let initial_supply = state.metadata.total_supply;
        state.mint(&recipient, 1_000_000).unwrap();
        assert_eq!(state.balance_of(&recipient), 1_000_000);
        assert_eq!(state.metadata.total_supply, initial_supply + 1_000_000);
    }

    #[test]
    fn test_burn() {
        let creator = SilverAddress::from_bytes(&[1u8; 64]).unwrap();
        let contract = SilverAddress::from_bytes(&[2u8; 64]).unwrap();
        let mut state = TokenState::new(
            "Test Token".to_string(),
            "TEST".to_string(),
            18,
            1_000_000_000_000_000_000_000u128,
            creator,
            contract,
            0,
        );

        let initial_supply = state.metadata.total_supply;
        state.burn(&creator, 500_000).unwrap();
        assert_eq!(state.balance_of(&creator), initial_supply - 500_000);
        assert_eq!(state.metadata.total_supply, initial_supply - 500_000);
    }
}
