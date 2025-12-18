//! Validator Rewards Distribution Module
//!
//! Manages validator rewards emission and distribution.
//! Unlike vesting (which is locked), rewards are emitted gradually and distributed
//! to validators based on their stake weight.
//!
//! Rewards Pool: 500M SBTC over 20 years = 25M SBTC/year
//!
//! Distribution Algorithm:
//! 1. Total annual emission: 25M SBTC (fixed)
//! 2. Calculate total stake across all active validators
//! 3. Distribute proportionally: validator_reward = (validator_stake / total_stake) * annual_emission
//! 4. Rewards are immediately available (not locked)
//! 5. Validator count can change dynamically - distribution adjusts automatically

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Validator rewards distribution configuration
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct RewardsConfig {
    /// Total rewards pool in MIST (500M SBTC)
    pub total_pool_mist: u128,

    /// Distribution period in years (20 years)
    pub distribution_years: u32,

    /// Annual emission in MIST (500M / 20 = 25M SBTC/year)
    pub annual_emission_mist: u128,

    /// Monthly emission in MIST (25M / 12)
    pub monthly_emission_mist: u128,
}

impl RewardsConfig {
    /// Create default rewards configuration
    /// Total: 500M SBTC over 20 years = 25M SBTC/year
    pub fn default() -> Self {
        let total_pool_mist = 500_000_000_000_000_000u128; // 500M SBTC in MIST
        let distribution_years = 20u32;
        let annual_emission_mist = total_pool_mist / (distribution_years as u128);
        let monthly_emission_mist = annual_emission_mist / 12u128;

        Self {
            total_pool_mist,
            distribution_years,
            annual_emission_mist,
            monthly_emission_mist,
        }
    }

    /// Verify configuration is valid
    pub fn verify(&self) -> Result<()> {
        if self.total_pool_mist == 0 {
            return Err(Error::InvalidData("Total pool must be greater than 0".to_string()));
        }

        if self.distribution_years == 0 {
            return Err(Error::InvalidData(
                "Distribution years must be greater than 0".to_string(),
            ));
        }

        let calculated_annual = self.total_pool_mist / (self.distribution_years as u128);
        if self.annual_emission_mist != calculated_annual {
            return Err(Error::InvalidData(
                "Annual emission does not match total pool / years".to_string(),
            ));
        }

        Ok(())
    }
}

/// Validator reward information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValidatorReward {
    /// Validator address
    pub validator_address: String,

    /// Validator stake in MIST
    pub stake_mist: u128,

    /// Commission rate (0.0 - 1.0)
    pub commission_rate: f64,

    /// Accumulated rewards in MIST
    pub accumulated_rewards_mist: u128,

    /// Claimed rewards in MIST
    pub claimed_rewards_mist: u128,

    /// Last reward distribution timestamp
    pub last_distribution_seconds: u64,

    /// Total delegated stake in MIST
    pub delegated_stake_mist: u128,

    /// Is validator active
    pub is_active: bool,
}

impl ValidatorReward {
    /// Create new validator reward entry
    pub fn new(
        validator_address: String,
        stake_mist: u128,
        commission_rate: f64,
        current_time_seconds: u64,
    ) -> Result<Self> {
        if commission_rate < 0.0 || commission_rate > 1.0 {
            return Err(Error::InvalidData(
                "Commission rate must be between 0.0 and 1.0".to_string(),
            ));
        }

        if stake_mist == 0 {
            return Err(Error::InvalidData("Stake must be greater than 0".to_string()));
        }

        Ok(Self {
            validator_address,
            stake_mist,
            commission_rate,
            accumulated_rewards_mist: 0,
            claimed_rewards_mist: 0,
            last_distribution_seconds: current_time_seconds,
            delegated_stake_mist: 0,
            is_active: true,
        })
    }

    /// Get total stake (own + delegated)
    pub fn get_total_stake(&self) -> u128 {
        self.stake_mist.saturating_add(self.delegated_stake_mist)
    }

    /// Get available rewards to claim
    pub fn get_available_rewards(&self) -> u128 {
        self.accumulated_rewards_mist.saturating_sub(self.claimed_rewards_mist)
    }

    /// Claim rewards
    pub fn claim_rewards(&mut self, amount_mist: u128) -> Result<u128> {
        let available = self.get_available_rewards();

        if amount_mist > available {
            return Err(Error::InsufficientVestedBalance(format!(
                "Cannot claim {} MIST, only {} available",
                amount_mist, available
            )));
        }

        self.claimed_rewards_mist += amount_mist;
        Ok(amount_mist)
    }

    /// Add delegation
    pub fn add_delegation(&mut self, amount_mist: u128) {
        self.delegated_stake_mist += amount_mist;
    }

    /// Remove delegation
    pub fn remove_delegation(&mut self, amount_mist: u128) -> Result<()> {
        if amount_mist > self.delegated_stake_mist {
            return Err(Error::InvalidData(
                "Cannot remove more delegation than exists".to_string(),
            ));
        }

        self.delegated_stake_mist = self.delegated_stake_mist.saturating_sub(amount_mist);
        Ok(())
    }
}

/// Delegator reward information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DelegatorReward {
    /// Delegator address
    pub delegator_address: String,

    /// Validator address they delegated to
    pub validator_address: String,

    /// Delegated stake in MIST
    pub delegated_stake_mist: u128,

    /// Accumulated rewards in MIST
    pub accumulated_rewards_mist: u128,

    /// Claimed rewards in MIST
    pub claimed_rewards_mist: u128,

    /// Last reward distribution timestamp
    pub last_distribution_seconds: u64,

    /// Is delegation active
    pub is_active: bool,
}

impl DelegatorReward {
    /// Create new delegator reward entry
    pub fn new(
        delegator_address: String,
        validator_address: String,
        delegated_stake_mist: u128,
        current_time_seconds: u64,
    ) -> Result<Self> {
        if delegated_stake_mist == 0 {
            return Err(Error::InvalidData(
                "Delegated stake must be greater than 0".to_string(),
            ));
        }

        Ok(Self {
            delegator_address,
            validator_address,
            delegated_stake_mist,
            accumulated_rewards_mist: 0,
            claimed_rewards_mist: 0,
            last_distribution_seconds: current_time_seconds,
            is_active: true,
        })
    }

    /// Get available rewards to claim
    pub fn get_available_rewards(&self) -> u128 {
        self.accumulated_rewards_mist.saturating_sub(self.claimed_rewards_mist)
    }

    /// Claim rewards
    pub fn claim_rewards(&mut self, amount_mist: u128) -> Result<u128> {
        let available = self.get_available_rewards();

        if amount_mist > available {
            return Err(Error::InsufficientVestedBalance(format!(
                "Cannot claim {} MIST, only {} available",
                amount_mist, available
            )));
        }

        self.claimed_rewards_mist += amount_mist;
        Ok(amount_mist)
    }
}

/// Rewards distribution record for a specific epoch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardDistribution {
    /// Epoch number
    pub epoch: u64,

    /// Distribution timestamp (Unix seconds)
    pub timestamp_seconds: u64,

    /// Total rewards distributed in MIST (fixed: 25M SBTC/year ÷ 12 months)
    pub total_rewards_mist: u128,

    /// Rewards by validator address (proportional to stake)
    pub validator_rewards: BTreeMap<String, u128>,

    /// Number of active validators at distribution time
    pub active_validators: usize,

    /// Total stake across all validators at distribution time
    pub total_stake_mist: u128,

    /// Distribution details for audit
    pub distribution_details: DistributionDetails,
}

/// Distribution calculation details for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributionDetails {
    /// Monthly emission amount (25M SBTC / 12)
    pub monthly_emission_mist: u128,

    /// Calculation method: "proportional_to_stake"
    pub calculation_method: String,

    /// Timestamp when calculation was performed
    pub calculated_at_seconds: u64,

    /// Total fees burned in this distribution period (MIST)
    pub fees_burned_mist: u128,
}

impl RewardDistribution {
    /// Create new reward distribution
    pub fn new(
        epoch: u64,
        timestamp_seconds: u64,
        monthly_emission_mist: u128,
    ) -> Self {
        Self {
            epoch,
            timestamp_seconds,
            total_rewards_mist: monthly_emission_mist,
            validator_rewards: BTreeMap::new(),
            active_validators: 0,
            total_stake_mist: 0,
            distribution_details: DistributionDetails {
                monthly_emission_mist,
                calculation_method: "proportional_to_stake".to_string(),
                calculated_at_seconds: timestamp_seconds,
                fees_burned_mist: 0,
            },
        }
    }
}

/// Rewards manager for validator rewards distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardsManager {
    /// Validator rewards indexed by address
    pub validators: BTreeMap<String, ValidatorReward>,

    /// Delegator rewards indexed by (delegator_address, validator_address)
    pub delegators: BTreeMap<(String, String), DelegatorReward>,

    /// Genesis time (Unix timestamp)
    pub genesis_time_seconds: u64,

    /// Rewards configuration (500M SBTC over 20 years)
    pub config: RewardsConfig,

    /// Emitted rewards so far in MIST
    pub emitted_rewards_mist: u128,

    /// Remaining rewards in pool in MIST
    pub remaining_pool_mist: u128,

    /// Distribution history
    pub distributions: Vec<RewardDistribution>,

    /// Current epoch (monthly)
    pub current_epoch: u64,

    /// Last distribution timestamp
    pub last_distribution_seconds: u64,
}

impl RewardsManager {
    /// Create new rewards manager with default configuration
    /// Configuration: 500M SBTC over 20 years = 25M SBTC/year = ~2.083M SBTC/month
    pub fn new(genesis_time_seconds: u64) -> Result<Self> {
        let config = RewardsConfig::default();
        config.verify()?;

        Ok(Self {
            validators: BTreeMap::new(),
            delegators: BTreeMap::new(),
            genesis_time_seconds,
            config,
            emitted_rewards_mist: 0,
            remaining_pool_mist: config.total_pool_mist,
            distributions: Vec::new(),
            current_epoch: 0,
            last_distribution_seconds: genesis_time_seconds,
        })
    }

    /// Create rewards manager with custom configuration
    pub fn with_config(genesis_time_seconds: u64, config: RewardsConfig) -> Result<Self> {
        config.verify()?;

        Ok(Self {
            validators: BTreeMap::new(),
            delegators: BTreeMap::new(),
            genesis_time_seconds,
            config,
            emitted_rewards_mist: 0,
            remaining_pool_mist: config.total_pool_mist,
            distributions: Vec::new(),
            current_epoch: 0,
            last_distribution_seconds: genesis_time_seconds,
        })
    }

    /// Register validator
    pub fn register_validator(
        &mut self,
        validator_address: String,
        stake_mist: u128,
        commission_rate: f64,
        current_time_seconds: u64,
    ) -> Result<()> {
        if self.validators.contains_key(&validator_address) {
            return Err(Error::InvalidData("Validator already registered".to_string()));
        }

        let reward = ValidatorReward::new(
            validator_address.clone(),
            stake_mist,
            commission_rate,
            current_time_seconds,
        )?;

        self.validators.insert(validator_address, reward);
        Ok(())
    }

    /// Get validator reward
    pub fn get_validator(&self, address: &str) -> Option<&ValidatorReward> {
        self.validators.get(address)
    }

    /// Get mutable validator reward
    pub fn get_validator_mut(&mut self, address: &str) -> Option<&mut ValidatorReward> {
        self.validators.get_mut(address)
    }



    /// Calculate total stake across all validators
    pub fn get_total_stake(&self) -> u128 {
        self.validators
            .values()
            .map(|v| v.get_total_stake())
            .sum()
    }

    /// Distribute rewards for current epoch
    /// 
    /// Distribution Algorithm:
    /// 1. Fixed monthly emission: 25M SBTC/year ÷ 12 = ~2.083M SBTC/month
    /// 2. Calculate total stake across all active validators (own + delegated)
    /// 3. For each validator: reward = (total_validator_stake / total_stake) * monthly_emission
    /// 4. Split reward between validator and delegators:
    ///    - Validator gets: reward * (1 - commission_rate)
    ///    - Delegators get: reward * commission_rate (distributed proportionally)
    /// 5. Rewards are immediately available (not locked)
    /// 6. Validator count can change - distribution adjusts automatically
    pub fn distribute_rewards(
        &mut self,
        current_time_seconds: u64,
    ) -> Result<RewardDistribution> {
        let monthly_emission = self.config.monthly_emission_mist;

        // Check if enough rewards remain
        if monthly_emission > self.remaining_pool_mist {
            return Err(Error::InvalidData(
                "Insufficient rewards remaining in pool".to_string(),
            ));
        }

        let mut distribution = RewardDistribution::new(
            self.current_epoch,
            current_time_seconds,
            monthly_emission,
        );

        // Get total stake across all active validators (own + delegated)
        let total_stake = self.get_total_stake();
        let active_validators = self.get_active_validator_count();

        distribution.active_validators = active_validators;
        distribution.total_stake_mist = total_stake;

        // Distribute rewards proportionally to total stake (own + delegated)
        if total_stake > 0 && active_validators > 0 {
            for (address, validator) in self.validators.iter_mut() {
                if !validator.is_active {
                    continue;
                }

                let validator_total_stake = validator.get_total_stake();
                
                // Calculate proportional reward based on total stake
                // total_reward = (validator_total_stake / total_stake) * monthly_emission
                let stake_percentage = (validator_total_stake as f64) / (total_stake as f64);
                let total_validator_reward = (monthly_emission as f64 * stake_percentage) as u128;

                // Split reward between validator and delegators
                // validator_share = total_reward * (1 - commission_rate)
                // delegator_share = total_reward * commission_rate
                let validator_share = (total_validator_reward as f64 * (1.0 - validator.commission_rate)) as u128;
                let delegator_share = total_validator_reward.saturating_sub(validator_share);

                // Add validator's own stake reward
                validator.accumulated_rewards_mist += validator_share;

                // Distribute delegator rewards proportionally to their stake
                if validator.delegated_stake_mist > 0 && delegator_share > 0 {
                    // Get all delegations for this validator
                    let delegations: Vec<_> = self
                        .delegators
                        .values()
                        .filter(|d| d.validator_address == *address && d.is_active)
                        .map(|d| (d.delegator_address.clone(), d.validator_address.clone(), d.delegated_stake_mist))
                        .collect();

                    if !delegations.is_empty() {
                        // Distribute delegator_share proportionally to each delegator's stake
                        for (delegator_addr, validator_addr, delegated_stake) in delegations {
                            let delegator_percentage =
                                (delegated_stake as f64) / (validator.delegated_stake_mist as f64);
                            let delegator_reward = (delegator_share as f64 * delegator_percentage) as u128;

                            // Update delegator's accumulated rewards
                            let key = (delegator_addr, validator_addr);
                            if let Some(delegator) = self.delegators.get_mut(&key) {
                                delegator.accumulated_rewards_mist += delegator_reward;
                                delegator.last_distribution_seconds = current_time_seconds;
                            }
                        }
                    }
                }

                validator.last_distribution_seconds = current_time_seconds;

                distribution
                    .validator_rewards
                    .insert(address.clone(), total_validator_reward);
            }
        } else if active_validators > 0 {
            // No stake yet - distribute equally among active validators
            let equal_share = monthly_emission / (active_validators as u128);
            
            for (address, validator) in self.validators.iter_mut() {
                if !validator.is_active {
                    continue;
                }

                validator.accumulated_rewards_mist += equal_share;
                validator.last_distribution_seconds = current_time_seconds;

                distribution
                    .validator_rewards
                    .insert(address.clone(), equal_share);
            }
        }

        // Update pool state
        self.emitted_rewards_mist += monthly_emission;
        self.remaining_pool_mist = self.remaining_pool_mist.saturating_sub(monthly_emission);
        self.distributions.push(distribution.clone());
        self.current_epoch += 1;
        self.last_distribution_seconds = current_time_seconds;

        Ok(distribution)
    }

    /// Add delegation to a validator
    pub fn add_delegation(
        &mut self,
        delegator_address: String,
        validator_address: String,
        delegated_stake_mist: u128,
        current_time_seconds: u64,
    ) -> Result<()> {
        // Verify validator exists and is active
        if !self.validators.contains_key(&validator_address) {
            return Err(Error::InvalidData("Validator not found".to_string()));
        }

        let key = (delegator_address.clone(), validator_address.clone());

        // Check if delegation already exists
        if self.delegators.contains_key(&key) {
            return Err(Error::InvalidData(
                "Delegation already exists for this delegator-validator pair".to_string(),
            ));
        }

        // Create delegator reward entry
        let delegator = DelegatorReward::new(
            delegator_address,
            validator_address.clone(),
            delegated_stake_mist,
            current_time_seconds,
        )?;

        // Add to delegators map
        self.delegators.insert(key, delegator);

        // Update validator's delegated stake
        if let Some(validator) = self.validators.get_mut(&validator_address) {
            validator.add_delegation(delegated_stake_mist);
        }

        Ok(())
    }

    /// Remove delegation from a validator
    pub fn remove_delegation(
        &mut self,
        delegator_address: &str,
        validator_address: &str,
        _current_time_seconds: u64,
    ) -> Result<u128> {
        let key = (delegator_address.to_string(), validator_address.to_string());

        // Get and remove delegator
        let delegator = self
            .delegators
            .remove(&key)
            .ok_or_else(|| Error::InvalidData("Delegation not found".to_string()))?;

        let delegated_amount = delegator.delegated_stake_mist;

        // Update validator's delegated stake
        if let Some(validator) = self.validators.get_mut(validator_address) {
            validator.remove_delegation(delegated_amount)?;
        }

        Ok(delegated_amount)
    }

    /// Get delegator rewards
    pub fn get_delegator(
        &self,
        delegator_address: &str,
        validator_address: &str,
    ) -> Option<&DelegatorReward> {
        let key = (delegator_address.to_string(), validator_address.to_string());
        self.delegators.get(&key)
    }

    /// Get mutable delegator rewards
    pub fn get_delegator_mut(
        &mut self,
        delegator_address: &str,
        validator_address: &str,
    ) -> Option<&mut DelegatorReward> {
        let key = (delegator_address.to_string(), validator_address.to_string());
        self.delegators.get_mut(&key)
    }

    /// Get all delegations for a delegator
    pub fn get_delegator_delegations(&self, delegator_address: &str) -> Vec<&DelegatorReward> {
        self.delegators
            .values()
            .filter(|d| d.delegator_address == delegator_address)
            .collect()
    }

    /// Get all delegations for a validator
    pub fn get_validator_delegations(&self, validator_address: &str) -> Vec<&DelegatorReward> {
        self.delegators
            .values()
            .filter(|d| d.validator_address == validator_address)
            .collect()
    }

    /// Claim validator rewards
    pub fn claim_validator_rewards(
        &mut self,
        validator_address: &str,
        amount_mist: u128,
    ) -> Result<u128> {
        if let Some(validator) = self.validators.get_mut(validator_address) {
            validator.claim_rewards(amount_mist)
        } else {
            Err(Error::InvalidData("Validator not found".to_string()))
        }
    }

    /// Claim delegator rewards
    pub fn claim_delegator_rewards(
        &mut self,
        delegator_address: &str,
        validator_address: &str,
        amount_mist: u128,
    ) -> Result<u128> {
        let key = (delegator_address.to_string(), validator_address.to_string());

        if let Some(delegator) = self.delegators.get_mut(&key) {
            delegator.claim_rewards(amount_mist)
        } else {
            Err(Error::InvalidData("Delegation not found".to_string()))
        }
    }

    /// Get total rewards distributed
    pub fn get_total_distributed(&self) -> u128 {
        self.distributions
            .iter()
            .map(|d| d.total_rewards_mist)
            .sum()
    }

    /// Get total fees burned
    pub fn get_total_burned(&self) -> u128 {
        self.distributions
            .iter()
            .map(|d| d.distribution_details.fees_burned_mist)
            .sum()
    }

    /// Get total delegated stake across all validators
    pub fn get_total_delegated_stake(&self) -> u128 {
        self.validators
            .values()
            .map(|v| v.delegated_stake_mist)
            .sum()
    }

    /// Get number of active delegators
    pub fn get_active_delegator_count(&self) -> usize {
        self.delegators.values().filter(|d| d.is_active).count()
    }

    /// Get total delegator rewards accumulated
    pub fn get_total_delegator_rewards(&self) -> u128 {
        self.delegators
            .values()
            .map(|d| d.accumulated_rewards_mist)
            .sum()
    }

    /// Get total delegator rewards claimed
    pub fn get_total_delegator_claimed(&self) -> u128 {
        self.delegators
            .values()
            .map(|d| d.claimed_rewards_mist)
            .sum()
    }

    /// Get rewards statistics
    pub fn get_statistics(&self) -> RewardsStatistics {
        let total_validator_accumulated: u128 = self
            .validators
            .values()
            .map(|v| v.accumulated_rewards_mist)
            .sum();

        let total_validator_claimed: u128 = self
            .validators
            .values()
            .map(|v| v.claimed_rewards_mist)
            .sum();

        let total_delegator_accumulated = self.get_total_delegator_rewards();
        let total_delegator_claimed = self.get_total_delegator_claimed();

        RewardsStatistics {
            total_emitted_mist: self.emitted_rewards_mist,
            remaining_pool_mist: self.remaining_pool_mist,
            total_validator_accumulated,
            total_validator_claimed,
            total_delegator_accumulated,
            total_delegator_claimed,
            active_validators: self.get_active_validator_count(),
            active_delegators: self.get_active_delegator_count(),
            total_validator_stake: self.get_total_stake(),
            total_delegated_stake: self.get_total_delegated_stake(),
            current_epoch: self.current_epoch,
        }
    }

    /// Get number of active validators
    pub fn get_active_validator_count(&self) -> usize {
        self.validators.values().filter(|v| v.is_active).count()
    }
}

/// Comprehensive rewards statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardsStatistics {
    /// Total rewards emitted from pool in MIST
    pub total_emitted_mist: u128,

    /// Remaining rewards in pool in MIST
    pub remaining_pool_mist: u128,

    /// Total rewards accumulated by validators in MIST
    pub total_validator_accumulated: u128,

    /// Total rewards claimed by validators in MIST
    pub total_validator_claimed: u128,

    /// Total rewards accumulated by delegators in MIST
    pub total_delegator_accumulated: u128,

    /// Total rewards claimed by delegators in MIST
    pub total_delegator_claimed: u128,

    /// Number of active validators
    pub active_validators: usize,

    /// Number of active delegators
    pub active_delegators: usize,

    /// Total validator stake in MIST
    pub total_validator_stake: u128,

    /// Total delegated stake in MIST
    pub total_delegated_stake: u128,

    /// Current epoch number
    pub current_epoch: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rewards_config() {
        let config = RewardsConfig::default();
        
        // 500M SBTC total
        assert_eq!(config.total_pool_mist, 500_000_000_000_000_000u128);
        
        // 20 years distribution
        assert_eq!(config.distribution_years, 20u32);
        
        // 25M SBTC/year
        assert_eq!(config.annual_emission_mist, 25_000_000_000_000_000u128);
        
        // ~2.083M SBTC/month
        assert_eq!(config.monthly_emission_mist, 2_083_333_333_333_333u128);
        
        // Verify configuration
        assert!(config.verify().is_ok());
    }

    #[test]
    fn test_validator_reward_creation() {
        let reward = ValidatorReward::new(
            "validator1".to_string(),
            2_500_000_000_000_000u128,
            0.05,
            1000000u64,
        );

        assert!(reward.is_ok());
        let reward = reward.unwrap();
        assert_eq!(reward.stake_mist, 2_500_000_000_000_000u128);
        assert_eq!(reward.commission_rate, 0.05);
        assert_eq!(reward.accumulated_rewards_mist, 0);
    }

    #[test]
    fn test_rewards_manager_creation() {
        let genesis_time = 1000000u64;
        let manager = RewardsManager::new(genesis_time);

        assert!(manager.is_ok());
        let manager = manager.unwrap();
        assert_eq!(manager.config.total_pool_mist, 500_000_000_000_000_000u128);
        assert_eq!(manager.config.distribution_years, 20u32);
    }

    #[test]
    fn test_proportional_distribution_two_validators() {
        let genesis_time = 1000000u64;
        let mut manager = RewardsManager::new(genesis_time).unwrap();

        // Register 2 validators with equal stake
        manager
            .register_validator(
                "validator1".to_string(),
                2_500_000_000_000_000u128,
                0.05,
                genesis_time,
            )
            .unwrap();

        manager
            .register_validator(
                "validator2".to_string(),
                2_500_000_000_000_000u128,
                0.05,
                genesis_time,
            )
            .unwrap();

        // Distribute rewards
        let distribution = manager.distribute_rewards(genesis_time + 30 * 24 * 3600);
        assert!(distribution.is_ok());

        let dist = distribution.unwrap();
        
        // Each validator should get 50% of monthly emission
        let validator1_reward = dist.validator_rewards.get("validator1").unwrap();
        let validator2_reward = dist.validator_rewards.get("validator2").unwrap();
        
        // Should be approximately equal (within rounding)
        let diff = if *validator1_reward > *validator2_reward {
            validator1_reward - validator2_reward
        } else {
            validator2_reward - validator1_reward
        };
        assert!(diff <= 1);
        
        // Total should equal monthly emission
        assert_eq!(
            validator1_reward + validator2_reward,
            manager.config.monthly_emission_mist
        );
    }

    #[test]
    fn test_proportional_distribution_unequal_stake() {
        let genesis_time = 1000000u64;
        let mut manager = RewardsManager::new(genesis_time).unwrap();

        // Register 2 validators with different stakes
        manager
            .register_validator(
                "validator1".to_string(),
                3_000_000_000_000_000u128,  // 60% stake
                0.05,
                genesis_time,
            )
            .unwrap();

        manager
            .register_validator(
                "validator2".to_string(),
                2_000_000_000_000_000u128,  // 40% stake
                0.05,
                genesis_time,
            )
            .unwrap();

        // Distribute rewards
        let distribution = manager.distribute_rewards(genesis_time + 30 * 24 * 3600);
        assert!(distribution.is_ok());

        let dist = distribution.unwrap();
        
        let validator1_reward = dist.validator_rewards.get("validator1").unwrap();
        let validator2_reward = dist.validator_rewards.get("validator2").unwrap();
        
        // Validator1 should get ~60% of rewards
        let expected_v1 = (manager.config.monthly_emission_mist as f64 * 0.6) as u128;
        let diff_v1 = if *validator1_reward > expected_v1 {
            validator1_reward - expected_v1
        } else {
            expected_v1 - validator1_reward
        };
        assert!(diff_v1 <= 1);
        
        // Validator2 should get ~40% of rewards
        let expected_v2 = (manager.config.monthly_emission_mist as f64 * 0.4) as u128;
        let diff_v2 = if *validator2_reward > expected_v2 {
            validator2_reward - expected_v2
        } else {
            expected_v2 - validator2_reward
        };
        assert!(diff_v2 <= 1);
    }

    #[test]
    fn test_dynamic_validator_addition() {
        let genesis_time = 1000000u64;
        let mut manager = RewardsManager::new(genesis_time).unwrap();

        // Start with 2 validators
        manager
            .register_validator(
                "validator1".to_string(),
                2_500_000_000_000_000u128,
                0.05,
                genesis_time,
            )
            .unwrap();

        manager
            .register_validator(
                "validator2".to_string(),
                2_500_000_000_000_000u128,
                0.05,
                genesis_time,
            )
            .unwrap();

        // First distribution
        let dist1 = manager.distribute_rewards(genesis_time + 30 * 24 * 3600).unwrap();
        assert_eq!(dist1.active_validators, 2);

        // Add third validator
        manager
            .register_validator(
                "validator3".to_string(),
                2_500_000_000_000_000u128,
                0.05,
                genesis_time,
            )
            .unwrap();

        // Second distribution with 3 validators
        let dist2 = manager.distribute_rewards(genesis_time + 60 * 24 * 3600).unwrap();
        assert_eq!(dist2.active_validators, 3);

        // Each validator should now get ~33.33% instead of 50%
        let v1_reward = dist2.validator_rewards.get("validator1").unwrap();
        let expected = (manager.config.monthly_emission_mist as f64 / 3.0) as u128;
        let diff = if *v1_reward > expected {
            v1_reward - expected
        } else {
            expected - v1_reward
        };
        assert!(diff <= 1);
    }

    #[test]
    fn test_claim_rewards() {
        let genesis_time = 1000000u64;
        let mut manager = RewardsManager::new(genesis_time).unwrap();

        manager
            .register_validator(
                "validator1".to_string(),
                2_500_000_000_000_000u128,
                0.05,
                genesis_time,
            )
            .unwrap();

        // Distribute rewards
        manager
            .distribute_rewards(genesis_time + 30 * 24 * 3600)
            .unwrap();

        // Claim rewards
        let validator = manager.get_validator("validator1").unwrap();
        let available = validator.get_available_rewards();
        assert!(available > 0);

        let claimed = manager
            .claim_validator_rewards("validator1", available)
            .unwrap();
        assert_eq!(claimed, available);
    }

    #[test]
    fn test_20_year_distribution() {
        let genesis_time = 1000000u64;
        let mut manager = RewardsManager::new(genesis_time).unwrap();

        manager
            .register_validator(
                "validator1".to_string(),
                10_000_000_000_000_000u128,
                0.05,
                genesis_time,
            )
            .unwrap();

        // Simulate 20 years of monthly distributions
        let mut total_distributed = 0u128;
        for month in 0..240 {  // 20 years * 12 months
            let distribution_time = genesis_time + (month as u64 * 30 * 24 * 3600);
            let dist = manager.distribute_rewards(distribution_time).unwrap();
            total_distributed += dist.total_rewards_mist;
        }

        // Should have distributed approximately 500M SBTC
        let expected = 500_000_000_000_000_000u128;
        let difference = if total_distributed > expected {
            total_distributed - expected
        } else {
            expected - total_distributed
        };
        
        // Allow small rounding error
        assert!(difference < 1_000_000_000u128);  // Less than 1 SBTC error
    }

    #[test]
    fn test_delegator_rewards_distribution() {
        let genesis_time = 1000000u64;
        let mut manager = RewardsManager::new(genesis_time).unwrap();

        // Register validator with 5% commission
        manager
            .register_validator(
                "validator1".to_string(),
                5_000_000_000_000_000u128,  // 5M SBTC
                0.05,  // 5% commission
                genesis_time,
            )
            .unwrap();

        // Add delegators
        manager
            .add_delegation(
                "delegator1".to_string(),
                "validator1".to_string(),
                3_000_000_000_000_000u128,  // 3M SBTC
                genesis_time,
            )
            .unwrap();

        manager
            .add_delegation(
                "delegator2".to_string(),
                "validator1".to_string(),
                2_000_000_000_000_000u128,  // 2M SBTC
                genesis_time,
            )
            .unwrap();

        // Distribute rewards
        let _dist = manager.distribute_rewards(genesis_time + 30 * 24 * 3600).unwrap();

        // Total stake: 5M (validator) + 3M (delegator1) + 2M (delegator2) = 10M SBTC
        // Monthly emission: ~2.083M SBTC
        // Validator share: 2.083M * (1 - 0.05) = 1.979M SBTC
        // Delegator share: 2.083M * 0.05 = 0.104M SBTC
        // Delegator1 gets: 0.104M * (3M / 5M) = 0.0624M SBTC
        // Delegator2 gets: 0.104M * (2M / 5M) = 0.0416M SBTC

        let validator = manager.get_validator("validator1").unwrap();
        let delegator1 = manager.get_delegator("delegator1", "validator1").unwrap();
        let delegator2 = manager.get_delegator("delegator2", "validator1").unwrap();

        // Verify rewards were distributed
        assert!(validator.accumulated_rewards_mist > 0);
        assert!(delegator1.accumulated_rewards_mist > 0);
        assert!(delegator2.accumulated_rewards_mist > 0);

        // Verify delegator1 gets more than delegator2 (60% vs 40% of delegator share)
        assert!(delegator1.accumulated_rewards_mist > delegator2.accumulated_rewards_mist);

        // Verify total equals monthly emission
        let total = validator.accumulated_rewards_mist
            + delegator1.accumulated_rewards_mist
            + delegator2.accumulated_rewards_mist;
        assert_eq!(total, manager.config.monthly_emission_mist);
    }

    #[test]
    fn test_delegator_claim_rewards() {
        let genesis_time = 1000000u64;
        let mut manager = RewardsManager::new(genesis_time).unwrap();

        manager
            .register_validator(
                "validator1".to_string(),
                5_000_000_000_000_000u128,
                0.05,
                genesis_time,
            )
            .unwrap();

        manager
            .add_delegation(
                "delegator1".to_string(),
                "validator1".to_string(),
                3_000_000_000_000_000u128,
                genesis_time,
            )
            .unwrap();

        // Distribute rewards
        manager
            .distribute_rewards(genesis_time + 30 * 24 * 3600)
            .unwrap();

        // Claim delegator rewards
        let delegator = manager.get_delegator("delegator1", "validator1").unwrap();
        let available = delegator.get_available_rewards();
        assert!(available > 0);

        let claimed = manager
            .claim_delegator_rewards("delegator1", "validator1", available)
            .unwrap();
        assert_eq!(claimed, available);

        // Verify claimed amount
        let delegator = manager.get_delegator("delegator1", "validator1").unwrap();
        assert_eq!(delegator.claimed_rewards_mist, available);
    }

    #[test]
    fn test_multiple_delegators_per_validator() {
        let genesis_time = 1000000u64;
        let mut manager = RewardsManager::new(genesis_time).unwrap();

        manager
            .register_validator(
                "validator1".to_string(),
                5_000_000_000_000_000u128,
                0.05,
                genesis_time,
            )
            .unwrap();

        // Add 5 delegators with different stakes
        for i in 1..=5 {
            manager
                .add_delegation(
                    format!("delegator{}", i),
                    "validator1".to_string(),
                    (i as u128) * 1_000_000_000_000_000u128,  // 1M, 2M, 3M, 4M, 5M
                    genesis_time,
                )
                .unwrap();
        }

        // Distribute rewards
        manager
            .distribute_rewards(genesis_time + 30 * 24 * 3600)
            .unwrap();

        // Verify all delegators received rewards
        for i in 1..=5 {
            let delegator = manager
                .get_delegator(&format!("delegator{}", i), "validator1")
                .unwrap();
            assert!(delegator.accumulated_rewards_mist > 0);
        }

        // Verify delegator5 gets more than delegator1 (5M vs 1M stake)
        let d1 = manager.get_delegator("delegator1", "validator1").unwrap();
        let d5 = manager.get_delegator("delegator5", "validator1").unwrap();
        assert!(d5.accumulated_rewards_mist > d1.accumulated_rewards_mist);
    }

    #[test]
    fn test_remove_delegation() {
        let genesis_time = 1000000u64;
        let mut manager = RewardsManager::new(genesis_time).unwrap();

        manager
            .register_validator(
                "validator1".to_string(),
                5_000_000_000_000_000u128,
                0.05,
                genesis_time,
            )
            .unwrap();

        manager
            .add_delegation(
                "delegator1".to_string(),
                "validator1".to_string(),
                3_000_000_000_000_000u128,
                genesis_time,
            )
            .unwrap();

        // Verify delegation exists
        assert!(manager
            .get_delegator("delegator1", "validator1")
            .is_some());

        // Remove delegation
        let removed = manager
            .remove_delegation("delegator1", "validator1", genesis_time)
            .unwrap();
        assert_eq!(removed, 3_000_000_000_000_000u128);

        // Verify delegation is removed
        assert!(manager
            .get_delegator("delegator1", "validator1")
            .is_none());

        // Verify validator's delegated stake is updated
        let validator = manager.get_validator("validator1").unwrap();
        assert_eq!(validator.delegated_stake_mist, 0);
    }

    #[test]
    fn test_rewards_statistics() {
        let genesis_time = 1000000u64;
        let mut manager = RewardsManager::new(genesis_time).unwrap();

        manager
            .register_validator(
                "validator1".to_string(),
                5_000_000_000_000_000u128,
                0.05,
                genesis_time,
            )
            .unwrap();

        manager
            .add_delegation(
                "delegator1".to_string(),
                "validator1".to_string(),
                3_000_000_000_000_000u128,
                genesis_time,
            )
            .unwrap();

        // Distribute rewards
        manager
            .distribute_rewards(genesis_time + 30 * 24 * 3600)
            .unwrap();

        // Get statistics
        let stats = manager.get_statistics();

        assert_eq!(stats.active_validators, 1);
        assert_eq!(stats.active_delegators, 1);
        assert_eq!(stats.total_validator_stake, 5_000_000_000_000_000u128);
        assert_eq!(stats.total_delegated_stake, 3_000_000_000_000_000u128);
        assert!(stats.total_validator_accumulated > 0);
        assert!(stats.total_delegator_accumulated > 0);
        assert_eq!(stats.current_epoch, 1);
    }
}
