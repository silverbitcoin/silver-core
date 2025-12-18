//! # Vesting Schedule Module
//!
//! Manages token vesting schedules for genesis accounts.
//! Tokens are locked until vesting conditions are met.
//!
//! This module provides:
//! - Vesting schedule creation and management
//! - Vesting calculation based on time
//! - Cliff period support
//! - Monthly unlock tracking
//! - Persistent vesting state

use crate::error::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Represents a single vesting schedule for an account
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VestingSchedule {
    /// Total amount to vest in MIST (1 SBTC = 1,000,000,000 MIST)
    pub total_amount_mist: u128,

    /// Amount already vested and available in MIST
    pub vested_amount_mist: u128,

    /// Genesis/start time (Unix timestamp in seconds)
    pub start_time_seconds: u64,

    /// Cliff period in seconds (0 = no cliff)
    pub cliff_seconds: u64,

    /// Total vesting duration in seconds (after cliff)
    pub duration_seconds: u64,

    /// Monthly vesting amount in MIST
    pub monthly_amount_mist: u128,

    /// Last time vesting was processed (Unix timestamp in seconds)
    pub last_vested_time_seconds: u64,

    /// Number of months vested so far
    pub months_vested: u32,

    /// Account address this schedule belongs to
    pub account_address: String,

    /// Human-readable name for this vesting account
    pub account_name: String,

    /// Whether this schedule is active
    pub is_active: bool,
}

impl VestingSchedule {
    /// Create a new vesting schedule
    ///
    /// # Arguments
    /// * `total_amount_mist` - Total amount to vest in MIST
    /// * `start_time_seconds` - Genesis/start time (Unix timestamp)
    /// * `cliff_months` - Cliff period in months (0 = no cliff)
    /// * `vesting_years` - Total vesting period in years (after cliff)
    /// * `account_address` - Address of the vesting account
    /// * `account_name` - Human-readable name
    ///
    /// # Returns
    /// A new VestingSchedule instance
    pub fn new(
        total_amount_mist: u128,
        start_time_seconds: u64,
        cliff_months: u32,
        vesting_years: u32,
        account_address: String,
        account_name: String,
    ) -> Result<Self> {
        if total_amount_mist == 0 {
            return Err(Error::InvalidVestingAmount);
        }

        if vesting_years == 0 {
            return Err(Error::InvalidVestingPeriod);
        }

        let cliff_seconds = (cliff_months as u64) * 30 * 24 * 3600; // Approximate month
        let duration_seconds = (vesting_years as u64) * 365 * 24 * 3600;
        let vesting_months = vesting_years * 12;
        let monthly_amount_mist = total_amount_mist / (vesting_months as u128);

        Ok(Self {
            total_amount_mist,
            vested_amount_mist: 0,
            start_time_seconds,
            cliff_seconds,
            duration_seconds,
            monthly_amount_mist,
            last_vested_time_seconds: start_time_seconds,
            months_vested: 0,
            account_address,
            account_name,
            is_active: true,
        })
    }

    /// Calculate total vested amount at a given timestamp
    ///
    /// # Arguments
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// Total amount vested in MIST
    pub fn calculate_vested_at(&self, current_time_seconds: u64) -> u128 {
        if !self.is_active {
            return self.vested_amount_mist;
        }

        // Before cliff, nothing is vested
        let cliff_end_time = self.start_time_seconds + self.cliff_seconds;
        if current_time_seconds < cliff_end_time {
            return 0;
        }

        // After cliff, calculate vested amount
        let vesting_end_time = cliff_end_time + self.duration_seconds;

        if current_time_seconds >= vesting_end_time {
            // Fully vested
            self.total_amount_mist
        } else {
            // Partially vested - calculate based on elapsed time
            let elapsed_seconds = current_time_seconds - cliff_end_time;
            let elapsed_months = elapsed_seconds / (30 * 24 * 3600); // Approximate month

            let vested = self.monthly_amount_mist * (elapsed_months as u128);
            vested.min(self.total_amount_mist)
        }
    }

    /// Get the amount available to unlock at current time
    ///
    /// # Arguments
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// Amount available to unlock in MIST
    pub fn get_available_unlock(&self, current_time_seconds: u64) -> u128 {
        let total_vested = self.calculate_vested_at(current_time_seconds);
        total_vested.saturating_sub(self.vested_amount_mist)
    }

    /// Get the locked amount at current time
    ///
    /// # Arguments
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// Amount still locked in MIST
    pub fn get_locked_amount(&self, current_time_seconds: u64) -> u128 {
        self.total_amount_mist.saturating_sub(self.calculate_vested_at(current_time_seconds))
    }

    /// Process vesting unlock at current time
    ///
    /// # Arguments
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// Amount unlocked in MIST
    pub fn process_unlock(&mut self, current_time_seconds: u64) -> Result<u128> {
        if !self.is_active {
            return Ok(0);
        }

        let available = self.get_available_unlock(current_time_seconds);

        if available > 0 {
            self.vested_amount_mist += available;
            self.last_vested_time_seconds = current_time_seconds;

            // Calculate months vested
            let cliff_end_time = self.start_time_seconds + self.cliff_seconds;
            if current_time_seconds >= cliff_end_time {
                let elapsed_seconds = current_time_seconds - cliff_end_time;
                self.months_vested = (elapsed_seconds / (30 * 24 * 3600)) as u32;
            }

            // Check if fully vested
            if self.vested_amount_mist >= self.total_amount_mist {
                self.is_active = false;
            }
        }

        Ok(available)
    }

    /// Check if fully vested at current time
    ///
    /// # Arguments
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// true if fully vested, false otherwise
    pub fn is_fully_vested(&self, current_time_seconds: u64) -> bool {
        self.calculate_vested_at(current_time_seconds) >= self.total_amount_mist
    }

    /// Get vesting progress percentage
    ///
    /// # Arguments
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// Percentage vested (0-100)
    pub fn get_vesting_percentage(&self, current_time_seconds: u64) -> f64 {
        if self.total_amount_mist == 0 {
            return 0.0;
        }

        let vested = self.calculate_vested_at(current_time_seconds);
        ((vested as f64) / (self.total_amount_mist as f64)) * 100.0
    }

    /// Get next unlock time
    ///
    /// # Returns
    /// Unix timestamp of next unlock, or None if fully vested
    pub fn get_next_unlock_time(&self) -> Option<u64> {
        if !self.is_active {
            return None;
        }

        let cliff_end_time = self.start_time_seconds + self.cliff_seconds;
        let vesting_end_time = cliff_end_time + self.duration_seconds;

        if self.last_vested_time_seconds >= vesting_end_time {
            return None;
        }

        // Next unlock is approximately 30 days from last vested time
        let next_unlock = self.last_vested_time_seconds + (30 * 24 * 3600);
        Some(next_unlock.min(vesting_end_time))
    }

    /// Get next unlock amount
    ///
    /// # Returns
    /// Amount to be unlocked at next unlock time in MIST
    pub fn get_next_unlock_amount(&self) -> u128 {
        if let Some(next_time) = self.get_next_unlock_time() {
            self.get_available_unlock(next_time)
        } else {
            0
        }
    }
}

/// Manages all vesting schedules for the blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingManager {
    /// Vesting schedules indexed by account address
    pub schedules: BTreeMap<String, VestingSchedule>,

    /// Genesis time (Unix timestamp)
    pub genesis_time_seconds: u64,

    /// Total amount locked across all schedules
    pub total_locked_mist: u128,

    /// Total amount vested across all schedules
    pub total_vested_mist: u128,
}

impl VestingManager {
    /// Create a new vesting manager
    ///
    /// # Arguments
    /// * `genesis_time_seconds` - Genesis time (Unix timestamp)
    ///
    /// # Returns
    /// A new VestingManager instance
    pub fn new(genesis_time_seconds: u64) -> Self {
        Self {
            schedules: BTreeMap::new(),
            genesis_time_seconds,
            total_locked_mist: 0,
            total_vested_mist: 0,
        }
    }

    /// Add a vesting schedule
    ///
    /// # Arguments
    /// * `address` - Account address
    /// * `total_amount_mist` - Total amount to vest
    /// * `cliff_months` - Cliff period in months
    /// * `vesting_years` - Vesting period in years
    /// * `account_name` - Human-readable name
    ///
    /// # Returns
    /// Result of adding the schedule
    pub fn add_schedule(
        &mut self,
        address: String,
        total_amount_mist: u128,
        cliff_months: u32,
        vesting_years: u32,
        account_name: String,
    ) -> Result<()> {
        if self.schedules.contains_key(&address) {
            return Err(Error::VestingScheduleAlreadyExists);
        }

        let schedule = VestingSchedule::new(
            total_amount_mist,
            self.genesis_time_seconds,
            cliff_months,
            vesting_years,
            address.clone(),
            account_name,
        )?;

        self.total_locked_mist += total_amount_mist;
        self.schedules.insert(address, schedule);

        Ok(())
    }

    /// Get vesting schedule for an address
    ///
    /// # Arguments
    /// * `address` - Account address
    ///
    /// # Returns
    /// Reference to the vesting schedule, or None if not found
    pub fn get_schedule(&self, address: &str) -> Option<&VestingSchedule> {
        self.schedules.get(address)
    }

    /// Get mutable vesting schedule for an address
    ///
    /// # Arguments
    /// * `address` - Account address
    ///
    /// # Returns
    /// Mutable reference to the vesting schedule, or None if not found
    pub fn get_schedule_mut(&mut self, address: &str) -> Option<&mut VestingSchedule> {
        self.schedules.get_mut(address)
    }

    /// Get vested amount for an address at current time
    ///
    /// # Arguments
    /// * `address` - Account address
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// Vested amount in MIST
    pub fn get_vested_amount(&self, address: &str, current_time_seconds: u64) -> u128 {
        self.schedules
            .get(address)
            .map(|s| s.calculate_vested_at(current_time_seconds))
            .unwrap_or(0)
    }

    /// Get locked amount for an address at current time
    ///
    /// # Arguments
    /// * `address` - Account address
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// Locked amount in MIST
    pub fn get_locked_amount(&self, address: &str, current_time_seconds: u64) -> u128 {
        self.schedules
            .get(address)
            .map(|s| s.get_locked_amount(current_time_seconds))
            .unwrap_or(0)
    }

    /// Get available unlock for an address at current time
    ///
    /// # Arguments
    /// * `address` - Account address
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// Available unlock amount in MIST
    pub fn get_available_unlock(&self, address: &str, current_time_seconds: u64) -> u128 {
        self.schedules
            .get(address)
            .map(|s| s.get_available_unlock(current_time_seconds))
            .unwrap_or(0)
    }

    /// Process unlock for an address at current time
    ///
    /// # Arguments
    /// * `address` - Account address
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// Amount unlocked in MIST
    pub fn process_unlock(&mut self, address: &str, current_time_seconds: u64) -> Result<u128> {
        if let Some(schedule) = self.schedules.get_mut(address) {
            let unlocked = schedule.process_unlock(current_time_seconds)?;
            self.total_vested_mist += unlocked;
            self.total_locked_mist = self.total_locked_mist.saturating_sub(unlocked);
            Ok(unlocked)
        } else {
            Ok(0)
        }
    }

    /// Process all pending unlocks at current time
    ///
    /// # Arguments
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// Total amount unlocked in MIST
    pub fn process_all_unlocks(&mut self, current_time_seconds: u64) -> Result<u128> {
        let mut total_unlocked = 0u128;

        let addresses: Vec<String> = self.schedules.keys().cloned().collect();

        for address in addresses {
            let unlocked = self.process_unlock(&address, current_time_seconds)?;
            total_unlocked += unlocked;
        }

        Ok(total_unlocked)
    }

    /// Check if address is fully vested at current time
    ///
    /// # Arguments
    /// * `address` - Account address
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// true if fully vested, false otherwise
    pub fn is_fully_vested(&self, address: &str, current_time_seconds: u64) -> bool {
        self.schedules
            .get(address)
            .map(|s| s.is_fully_vested(current_time_seconds))
            .unwrap_or(true)
    }

    /// Get vesting progress for an address
    ///
    /// # Arguments
    /// * `address` - Account address
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// Vesting progress percentage (0-100)
    pub fn get_vesting_percentage(&self, address: &str, current_time_seconds: u64) -> f64 {
        self.schedules
            .get(address)
            .map(|s| s.get_vesting_percentage(current_time_seconds))
            .unwrap_or(0.0)
    }

    /// Get all vesting schedules
    ///
    /// # Returns
    /// Iterator over all vesting schedules
    pub fn get_all_schedules(&self) -> impl Iterator<Item = (&String, &VestingSchedule)> {
        self.schedules.iter()
    }

    /// Get total vesting statistics at current time
    ///
    /// # Arguments
    /// * `current_time_seconds` - Current time (Unix timestamp)
    ///
    /// # Returns
    /// Tuple of (total_vested, total_locked, total_amount)
    pub fn get_statistics(&self, current_time_seconds: u64) -> (u128, u128, u128) {
        let mut total_vested = 0u128;
        let mut total_locked = 0u128;
        let mut total_amount = 0u128;

        for schedule in self.schedules.values() {
            let vested = schedule.calculate_vested_at(current_time_seconds);
            let locked = schedule.get_locked_amount(current_time_seconds);

            total_vested += vested;
            total_locked += locked;
            total_amount += schedule.total_amount_mist;
        }

        (total_vested, total_locked, total_amount)
    }

    /// Get number of active vesting schedules
    ///
    /// # Returns
    /// Number of active schedules
    pub fn get_active_count(&self) -> usize {
        self.schedules.values().filter(|s| s.is_active).count()
    }

    /// Get number of completed vesting schedules
    ///
    /// # Returns
    /// Number of completed schedules
    pub fn get_completed_count(&self) -> usize {
        self.schedules.values().filter(|s| !s.is_active).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vesting_schedule_creation() {
        let genesis_time = 1000000u64;
        let schedule = VestingSchedule::new(
            1_000_000_000_000_000u128, // 1M SBTC in MIST
            genesis_time,
            0,  // No cliff
            1,  // 1 year vesting
            "test_address".to_string(),
            "Test Account".to_string(),
        );

        assert!(schedule.is_ok());
        let schedule = schedule.unwrap();
        assert_eq!(schedule.total_amount_mist, 1_000_000_000_000_000u128);
        assert_eq!(schedule.vested_amount_mist, 0);
        assert_eq!(schedule.cliff_seconds, 0);
    }

    #[test]
    fn test_vesting_calculation_no_cliff() {
        let genesis_time = 1000000u64;
        let schedule = VestingSchedule::new(
            1_200_000_000_000_000u128, // 1.2M SBTC
            genesis_time,
            0,  // No cliff
            1,  // 1 year vesting (12 months)
            "test_address".to_string(),
            "Test Account".to_string(),
        )
        .unwrap();

        // Before vesting period
        assert_eq!(schedule.calculate_vested_at(genesis_time), 0);

        // After 6 months
        let six_months = genesis_time + (6 * 30 * 24 * 3600);
        let vested_6m = schedule.calculate_vested_at(six_months);
        assert!(vested_6m > 0);
        assert!(vested_6m < 1_200_000_000_000_000u128);

        // After 1 year (fully vested)
        let one_year = genesis_time + (365 * 24 * 3600);
        assert_eq!(schedule.calculate_vested_at(one_year), 1_200_000_000_000_000u128);
    }

    #[test]
    fn test_vesting_with_cliff() {
        let genesis_time = 1000000u64;
        let schedule = VestingSchedule::new(
            1_000_000_000_000_000u128,
            genesis_time,
            12, // 12 month cliff
            4,  // 4 year total vesting
            "test_address".to_string(),
            "Test Account".to_string(),
        )
        .unwrap();

        // Before cliff
        let before_cliff = genesis_time + (6 * 30 * 24 * 3600);
        assert_eq!(schedule.calculate_vested_at(before_cliff), 0);

        // After cliff
        let after_cliff = genesis_time + (13 * 30 * 24 * 3600);
        assert!(schedule.calculate_vested_at(after_cliff) > 0);
    }

    #[test]
    fn test_vesting_manager() {
        let genesis_time = 1000000u64;
        let mut manager = VestingManager::new(genesis_time);

        manager
            .add_schedule(
                "address1".to_string(),
                1_000_000_000_000_000u128,
                0,
                1,
                "Account 1".to_string(),
            )
            .unwrap();

        manager
            .add_schedule(
                "address2".to_string(),
                2_000_000_000_000_000u128,
                6,
                2,
                "Account 2".to_string(),
            )
            .unwrap();

        assert_eq!(manager.schedules.len(), 2);
        assert_eq!(manager.total_locked_mist, 3_000_000_000_000_000u128);
    }

    #[test]
    fn test_process_unlock() {
        let genesis_time = 1000000u64;
        let mut schedule = VestingSchedule::new(
            1_200_000_000_000_000u128,
            genesis_time,
            0,
            1,
            "test_address".to_string(),
            "Test Account".to_string(),
        )
        .unwrap();

        // Process unlock after 6 months
        let six_months = genesis_time + (6 * 30 * 24 * 3600);
        let unlocked = schedule.process_unlock(six_months).unwrap();

        assert!(unlocked > 0);
        assert_eq!(schedule.vested_amount_mist, unlocked);
    }
}
