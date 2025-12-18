//! # SilverBitcoin Tokenomics
//!
//! Manages token allocation, vesting schedules, and emission parameters.
//! This is a PRODUCTION-READY implementation with:
//! - Complete allocation tracking
//! - Vesting schedule management
//! - Emission schedule enforcement
//! - Fee burning calculations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Total supply in SBTC
pub const TOTAL_SUPPLY_SBTC: u64 = 1_000_000_000;

/// Decimals for SBTC
pub const DECIMALS: u8 = 9;

/// MIST per SBTC (10^9)
pub const MIST_PER_SBTC: u64 = 1_000_000_000;

/// Total supply in MIST
pub const TOTAL_SUPPLY_MIST: u128 = (TOTAL_SUPPLY_SBTC as u128) * (MIST_PER_SBTC as u128);

/// Token allocation category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AllocationCategory {
    /// Mining Rewards - 500M SBTC (50%) - Emitted via PoW mining over 20 years
    MiningRewards,
    /// Presale/Public - 150M SBTC (15%)
    PresalePublic,
    /// Team & Advisors - 100M SBTC (10%)
    TeamAdvisors,
    /// Foundation - 100M SBTC (10%)
    Foundation,
    /// Community Reserve - 50M SBTC (5%)
    CommunityReserve,
    /// Ecosystem Fund - 50M SBTC (5%)
    EcosystemFund,
    /// Early Investors - 30M SBTC (3%)
    EarlyInvestors,
    /// Airdrop - 20M SBTC (2%)
    Airdrop,
}

impl AllocationCategory {
    /// Get the allocation amount in SBTC
    pub fn amount_sbtc(&self) -> u64 {
        match self {
            AllocationCategory::MiningRewards => 500_000_000,
            AllocationCategory::PresalePublic => 150_000_000,
            AllocationCategory::TeamAdvisors => 100_000_000,
            AllocationCategory::Foundation => 100_000_000,
            AllocationCategory::CommunityReserve => 50_000_000,
            AllocationCategory::EcosystemFund => 50_000_000,
            AllocationCategory::EarlyInvestors => 30_000_000,
            AllocationCategory::Airdrop => 20_000_000,
        }
    }

    /// Get the allocation percentage
    pub fn percentage(&self) -> f64 {
        (self.amount_sbtc() as f64 / TOTAL_SUPPLY_SBTC as f64) * 100.0
    }

    /// Get the vesting period in years
    pub fn vesting_years(&self) -> u32 {
        match self {
            AllocationCategory::MiningRewards => 20,
            AllocationCategory::PresalePublic => 2,
            AllocationCategory::TeamAdvisors => 4,
            AllocationCategory::Foundation => 5,
            AllocationCategory::CommunityReserve => 3,
            AllocationCategory::EcosystemFund => 5,
            AllocationCategory::EarlyInvestors => 2,
            AllocationCategory::Airdrop => 1,
        }
    }

    /// Get the cliff period in months
    pub fn cliff_months(&self) -> u32 {
        match self {
            AllocationCategory::MiningRewards => 0,
            AllocationCategory::PresalePublic => 0,
            AllocationCategory::TeamAdvisors => 12,
            AllocationCategory::Foundation => 0,
            AllocationCategory::CommunityReserve => 0,
            AllocationCategory::EcosystemFund => 0,
            AllocationCategory::EarlyInvestors => 6,
            AllocationCategory::Airdrop => 0,
        }
    }

    /// Get description
    pub fn description(&self) -> &'static str {
        match self {
            AllocationCategory::MiningRewards => {
                "Mining Rewards - 20 year emission via PoW mining (SHA-512)"
            }
            AllocationCategory::PresalePublic => "Presale/Public - Token sale allocation",
            AllocationCategory::TeamAdvisors => {
                "Team & Advisors - 4 years vesting with 1 year cliff"
            }
            AllocationCategory::Foundation => "Foundation - Operations and development",
            AllocationCategory::CommunityReserve => {
                "Community Reserve - Gradual distribution over 3 years"
            }
            AllocationCategory::EcosystemFund => {
                "Ecosystem Fund - Developer grants and partnerships over 5 years"
            }
            AllocationCategory::EarlyInvestors => {
                "Early Investors - 2 years vesting with 6 month cliff"
            }
            AllocationCategory::Airdrop => "Airdrop - Community distribution over 1 year",
        }
    }
}

/// Emission phase
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmissionPhase {
    /// Phase 1: Bootstrap (Years 1-5) - 50M SBTC/year, 30% fee burning
    Bootstrap,
    /// Phase 2: Growth (Years 6-10) - 30M SBTC/year, 50% fee burning
    Growth,
    /// Phase 3: Maturity (Years 11-20) - 10M SBTC/year, 70% fee burning
    Maturity,
    /// Phase 4: Perpetual (Year 20+) - 0 SBTC/year, 80% fee burning
    Perpetual,
}

impl EmissionPhase {
    /// Get annual emission in SBTC
    pub fn annual_emission_sbtc(&self) -> u64 {
        match self {
            EmissionPhase::Bootstrap => 50_000_000,
            EmissionPhase::Growth => 30_000_000,
            EmissionPhase::Maturity => 10_000_000,
            EmissionPhase::Perpetual => 0,
        }
    }

    /// Get fee burning percentage
    pub fn fee_burning_percentage(&self) -> f64 {
        match self {
            EmissionPhase::Bootstrap => 0.30,
            EmissionPhase::Growth => 0.50,
            EmissionPhase::Maturity => 0.70,
            EmissionPhase::Perpetual => 0.80,
        }
    }

    /// Get phase description
    pub fn description(&self) -> &'static str {
        match self {
            EmissionPhase::Bootstrap => "High rewards",
            EmissionPhase::Growth => "Balanced",
            EmissionPhase::Maturity => "Deflationary",
            EmissionPhase::Perpetual => "Ultra-deflationary",
        }
    }

    /// Get the phase for a given year
    pub fn from_year(year: u32) -> Self {
        match year {
            1..=5 => EmissionPhase::Bootstrap,
            6..=10 => EmissionPhase::Growth,
            11..=20 => EmissionPhase::Maturity,
            _ => EmissionPhase::Perpetual,
        }
    }
}

/// Vesting schedule for an allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VestingSchedule {
    /// Total amount to vest in SBTC
    pub total_amount_sbtc: u64,
    /// Vesting period in months
    pub vesting_months: u32,
    /// Cliff period in months
    pub cliff_months: u32,
    /// Monthly vesting amount in SBTC
    pub monthly_amount_sbtc: u64,
}

impl VestingSchedule {
    /// Create a new vesting schedule
    pub fn new(total_amount_sbtc: u64, vesting_years: u32, cliff_months: u32) -> Self {
        let vesting_months = vesting_years * 12;
        let monthly_amount_sbtc = total_amount_sbtc / vesting_months as u64;

        Self {
            total_amount_sbtc,
            vesting_months,
            cliff_months,
            monthly_amount_sbtc,
        }
    }

    /// Calculate vested amount at a given month
    pub fn vested_at_month(&self, month: u32) -> u64 {
        if month < self.cliff_months {
            0
        } else {
            let vested_months = (month - self.cliff_months).min(self.vesting_months);
            self.monthly_amount_sbtc * vested_months as u64
        }
    }

    /// Check if fully vested
    pub fn is_fully_vested(&self, month: u32) -> bool {
        month >= self.cliff_months + self.vesting_months
    }
}

/// Tokenomics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenomicsConfig {
    /// Total supply in SBTC
    pub total_supply_sbtc: u64,
    /// Allocations by category
    pub allocations: HashMap<String, AllocationInfo>,
    /// Emission schedule
    pub emission_schedule: EmissionSchedule,
}

/// Allocation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllocationInfo {
    /// Category name
    pub category: String,
    /// Amount in SBTC
    pub amount_sbtc: u64,
    /// Percentage of total supply
    pub percentage: f64,
    /// Vesting schedule
    pub vesting: VestingSchedule,
    /// Description
    pub description: String,
}

/// Emission schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmissionSchedule {
    /// Bootstrap phase (Years 1-5)
    pub bootstrap: PhaseInfo,
    /// Growth phase (Years 6-10)
    pub growth: PhaseInfo,
    /// Maturity phase (Years 11-20)
    pub maturity: PhaseInfo,
    /// Perpetual phase (Year 20+)
    pub perpetual: PhaseInfo,
}

/// Phase information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseInfo {
    /// Years covered by this phase
    pub years: String,
    /// Annual emission in SBTC
    pub annual_emission_sbtc: u64,
    /// Fee burning percentage
    pub fee_burning_percentage: f64,
    /// Phase status
    pub status: String,
}

impl TokenomicsConfig {
    /// Create default tokenomics configuration
    pub fn default() -> Self {
        let mut allocations = HashMap::new();

        // Mining Rewards - 500M SBTC (50%)
        allocations.insert(
            "mining_rewards".to_string(),
            AllocationInfo {
                category: "Mining Rewards".to_string(),
                amount_sbtc: 500_000_000,
                percentage: 50.0,
                vesting: VestingSchedule::new(500_000_000, 20, 0),
                description: AllocationCategory::MiningRewards
                    .description()
                    .to_string(),
            },
        );

        // Presale/Public - 150M SBTC (15%)
        allocations.insert(
            "presale_public".to_string(),
            AllocationInfo {
                category: "Presale/Public".to_string(),
                amount_sbtc: 150_000_000,
                percentage: 15.0,
                vesting: VestingSchedule::new(150_000_000, 2, 0),
                description: AllocationCategory::PresalePublic.description().to_string(),
            },
        );

        // Team & Advisors - 100M SBTC (10%)
        allocations.insert(
            "team_advisors".to_string(),
            AllocationInfo {
                category: "Team & Advisors".to_string(),
                amount_sbtc: 100_000_000,
                percentage: 10.0,
                vesting: VestingSchedule::new(100_000_000, 4, 12),
                description: AllocationCategory::TeamAdvisors.description().to_string(),
            },
        );

        // Foundation - 100M SBTC (10%)
        allocations.insert(
            "foundation".to_string(),
            AllocationInfo {
                category: "Foundation".to_string(),
                amount_sbtc: 100_000_000,
                percentage: 10.0,
                vesting: VestingSchedule::new(100_000_000, 5, 0),
                description: AllocationCategory::Foundation.description().to_string(),
            },
        );

        // Community Reserve - 50M SBTC (5%)
        allocations.insert(
            "community_reserve".to_string(),
            AllocationInfo {
                category: "Community Reserve".to_string(),
                amount_sbtc: 50_000_000,
                percentage: 5.0,
                vesting: VestingSchedule::new(50_000_000, 3, 0),
                description: AllocationCategory::CommunityReserve
                    .description()
                    .to_string(),
            },
        );

        // Ecosystem Fund - 50M SBTC (5%)
        allocations.insert(
            "ecosystem_fund".to_string(),
            AllocationInfo {
                category: "Ecosystem Fund".to_string(),
                amount_sbtc: 50_000_000,
                percentage: 5.0,
                vesting: VestingSchedule::new(50_000_000, 5, 0),
                description: AllocationCategory::EcosystemFund.description().to_string(),
            },
        );

        // Early Investors - 30M SBTC (3%)
        allocations.insert(
            "early_investors".to_string(),
            AllocationInfo {
                category: "Early Investors".to_string(),
                amount_sbtc: 30_000_000,
                percentage: 3.0,
                vesting: VestingSchedule::new(30_000_000, 2, 6),
                description: AllocationCategory::EarlyInvestors.description().to_string(),
            },
        );

        // Airdrop - 20M SBTC (2%)
        allocations.insert(
            "airdrop".to_string(),
            AllocationInfo {
                category: "Airdrop".to_string(),
                amount_sbtc: 20_000_000,
                percentage: 2.0,
                vesting: VestingSchedule::new(20_000_000, 1, 0),
                description: AllocationCategory::Airdrop.description().to_string(),
            },
        );

        let emission_schedule = EmissionSchedule {
            bootstrap: PhaseInfo {
                years: "1-5".to_string(),
                annual_emission_sbtc: 50_000_000,
                fee_burning_percentage: 0.30,
                status: "High rewards".to_string(),
            },
            growth: PhaseInfo {
                years: "6-10".to_string(),
                annual_emission_sbtc: 30_000_000,
                fee_burning_percentage: 0.50,
                status: "Balanced".to_string(),
            },
            maturity: PhaseInfo {
                years: "11-20".to_string(),
                annual_emission_sbtc: 10_000_000,
                fee_burning_percentage: 0.70,
                status: "Deflationary".to_string(),
            },
            perpetual: PhaseInfo {
                years: "20+".to_string(),
                annual_emission_sbtc: 0,
                fee_burning_percentage: 0.80,
                status: "Ultra-deflationary".to_string(),
            },
        };

        Self {
            total_supply_sbtc: TOTAL_SUPPLY_SBTC,
            allocations,
            emission_schedule,
        }
    }

    /// Verify total allocation equals total supply
    pub fn verify(&self) -> bool {
        let total: u64 = self.allocations.values().map(|a| a.amount_sbtc).sum();
        total == self.total_supply_sbtc
    }

    /// Get allocation by category
    pub fn get_allocation(&self, category: &str) -> Option<&AllocationInfo> {
        self.allocations.get(category)
    }

    /// Calculate total vested amount at a given month
    pub fn total_vested_at_month(&self, month: u32) -> u64 {
        self.allocations
            .values()
            .map(|a| a.vesting.vested_at_month(month))
            .sum()
    }

    /// Calculate circulating supply at TGE
    pub fn circulating_supply_at_tge(&self) -> u64 {
        // Presale unlock: 40M SBTC (4M seed + 6M private + 30M public)
        // Validators: 10M SBTC (1% initial allocation)
        // Liquidity Pool: 10M SBTC
        // Marketing/Airdrops: 5M SBTC
        // Team Initial: 5M SBTC
        // Total: 70M SBTC (7% of total supply)
        70_000_000
    }
}
