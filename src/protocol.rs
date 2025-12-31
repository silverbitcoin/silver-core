//! Protocol version and upgrade types
//!
//! This module defines types for protocol versioning and upgrade coordination.

use crate::{Error, Result, Signature, SilverAddress, SnapshotDigest};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha512};
use std::fmt;

/// Protocol version structure
///
/// Represents a specific version of the SilverBitcoin protocol.
/// Major version changes indicate breaking changes, while minor
/// version changes are backward compatible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ProtocolVersion {
    /// Major version number (breaking changes)
    pub major: u64,

    /// Minor version number (backward compatible changes)
    pub minor: u64,
}

impl ProtocolVersion {
    /// Create a new protocol version
    pub const fn new(major: u64, minor: u64) -> Self {
        Self { major, minor }
    }

    /// Get the current protocol version
    pub const fn current() -> Self {
        Self::new(1, 0)
    }

    /// Check if this version is compatible with another version
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        // Same major version means compatible
        self.major == other.major
    }

    /// Check if this is a breaking upgrade from another version
    pub fn is_breaking_upgrade_from(&self, other: &Self) -> bool {
        self.major > other.major
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "v{}.{}", self.major, self.minor)
    }
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::current()
    }
}

/// Feature flags for protocol features
///
/// Allows gradual rollout of new features and backward compatibility
/// during protocol upgrades.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeatureFlags {
    /// Enabled feature names
    pub enabled: Vec<String>,
}

impl FeatureFlags {
    /// Create empty feature flags
    pub fn new() -> Self {
        Self {
            enabled: Vec::new(),
        }
    }

    /// Create feature flags with specific features enabled
    pub fn with_features(features: Vec<String>) -> Self {
        Self { enabled: features }
    }

    /// Check if a feature is enabled
    pub fn is_enabled(&self, feature: &str) -> bool {
        self.enabled.iter().any(|f| f == feature)
    }

    /// Enable a feature
    pub fn enable(&mut self, feature: String) {
        if !self.is_enabled(&feature) {
            self.enabled.push(feature);
        }
    }

    /// Disable a feature
    pub fn disable(&mut self, feature: &str) {
        self.enabled.retain(|f| f != feature);
    }
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self::new()
    }
}

/// Protocol upgrade proposal
///
/// Represents a proposed upgrade to the protocol that requires
/// validator approval before activation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeProposal {
    /// Unique proposal ID
    pub proposal_id: ProposalID,

    /// New protocol version
    pub new_version: ProtocolVersion,

    /// Feature flags for the new version
    pub feature_flags: FeatureFlags,

    /// Cycle at which to activate if approved
    pub activation_cycle: u64,

    /// Proposer address
    pub proposer: SilverAddress,

    /// Proposal description
    pub description: String,

    /// Unix timestamp when proposal was created (milliseconds)
    pub created_at: u64,

    /// Voting deadline (cycle number)
    pub voting_deadline: u64,
}

impl UpgradeProposal {
    /// Create a new upgrade proposal
    pub fn new(
        new_version: ProtocolVersion,
        feature_flags: FeatureFlags,
        activation_cycle: u64,
        proposer: SilverAddress,
        description: String,
        created_at: u64,
        voting_deadline: u64,
    ) -> Self {
        let proposal_id = ProposalID::compute(
            &new_version,
            &feature_flags,
            activation_cycle,
            &proposer,
            created_at,
        );

        Self {
            proposal_id,
            new_version,
            feature_flags,
            activation_cycle,
            proposer,
            description,
            created_at,
            voting_deadline,
        }
    }

    /// Validate proposal structure
    pub fn validate(&self, current_cycle: u64) -> Result<()> {
        // Activation must be in the future
        if self.activation_cycle <= current_cycle {
            return Err(Error::InvalidData(format!(
                "Activation cycle {} must be after current cycle {}",
                self.activation_cycle, current_cycle
            )));
        }

        // Voting deadline must be before activation
        if self.voting_deadline >= self.activation_cycle {
            return Err(Error::InvalidData(format!(
                "Voting deadline {} must be before activation cycle {}",
                self.voting_deadline, self.activation_cycle
            )));
        }

        // Description cannot be empty
        if self.description.is_empty() {
            return Err(Error::InvalidData(
                "Proposal description cannot be empty".to_string(),
            ));
        }

        Ok(())
    }

    /// Check if voting is still open
    pub fn is_voting_open(&self, current_cycle: u64) -> bool {
        current_cycle <= self.voting_deadline
    }

    /// Check if proposal is ready for activation
    pub fn is_ready_for_activation(&self, current_cycle: u64) -> bool {
        current_cycle >= self.activation_cycle
    }
}

impl fmt::Display for UpgradeProposal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Proposal {{ id: {}, version: {}, activation: cycle {} }}",
            self.proposal_id, self.new_version, self.activation_cycle
        )
    }
}

/// Proposal ID (512-bit Blake3 hash)
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ProposalID(pub [u8; 64]);

// Implement Serialize/Deserialize for ProposalID
impl Serialize for ProposalID {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bytes(&self.0)
    }
}

impl<'de> Deserialize<'de> for ProposalID {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = ProposalID;

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
                Ok(ProposalID(arr))
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
                Ok(ProposalID(arr))
            }
        }

        deserializer.deserialize_bytes(Visitor)
    }
}

impl ProposalID {
    /// Create a new proposal ID
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

    /// Compute proposal ID from proposal data
    pub fn compute(
        version: &ProtocolVersion,
        feature_flags: &FeatureFlags,
        activation_cycle: u64,
        proposer: &SilverAddress,
        created_at: u64,
    ) -> Self {
        let mut hasher = Sha512::new();

        hasher.update(version.major.to_le_bytes());
        hasher.update(version.minor.to_le_bytes());

        for feature in &feature_flags.enabled {
            hasher.update(feature.as_bytes());
        }

        hasher.update(activation_cycle.to_le_bytes());
        hasher.update(proposer.as_bytes());
        hasher.update(created_at.to_le_bytes());

        let mut output = [0u8; 64];
        output.copy_from_slice(&hasher.finalize());
        Self(output)
    }
}

impl fmt::Debug for ProposalID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProposalID({}...)", &self.to_hex()[..16])
    }
}

impl fmt::Display for ProposalID {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_hex()[..16])
    }
}

/// Validator vote on an upgrade proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeVote {
    /// Proposal being voted on
    pub proposal_id: ProposalID,

    /// Validator address
    pub validator: SilverAddress,

    /// Vote (true = approve, false = reject)
    pub approve: bool,

    /// Validator's stake weight at time of vote
    pub stake_weight: u64,

    /// Signature from validator
    pub signature: Signature,

    /// Unix timestamp when vote was cast (milliseconds)
    pub timestamp: u64,
}

impl UpgradeVote {
    /// Create a new upgrade vote
    pub fn new(
        proposal_id: ProposalID,
        validator: SilverAddress,
        approve: bool,
        stake_weight: u64,
        signature: Signature,
        timestamp: u64,
    ) -> Self {
        Self {
            proposal_id,
            validator,
            approve,
            stake_weight,
            signature,
            timestamp,
        }
    }

    /// Validate vote structure
    pub fn validate(&self) -> Result<()> {
        if self.stake_weight == 0 {
            return Err(Error::InvalidData(
                "Vote stake weight cannot be zero".to_string(),
            ));
        }

        Ok(())
    }
}

/// Voting results for an upgrade proposal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VotingResults {
    /// Proposal being voted on
    pub proposal_id: ProposalID,

    /// Total stake weight that voted to approve
    pub approve_stake: u64,

    /// Total stake weight that voted to reject
    pub reject_stake: u64,

    /// Total stake weight in the validator set
    pub total_stake: u64,

    /// Individual votes
    pub votes: Vec<UpgradeVote>,
}

impl VotingResults {
    /// Create new voting results
    pub fn new(proposal_id: ProposalID, total_stake: u64) -> Self {
        Self {
            proposal_id,
            approve_stake: 0,
            reject_stake: 0,
            total_stake,
            votes: Vec::new(),
        }
    }

    /// Add a vote to the results
    pub fn add_vote(&mut self, vote: UpgradeVote) -> Result<()> {
        // Check if validator already voted
        if self.votes.iter().any(|v| v.validator == vote.validator) {
            return Err(Error::InvalidData(format!(
                "Validator {} already voted",
                vote.validator
            )));
        }

        // Update stake counts
        if vote.approve {
            self.approve_stake += vote.stake_weight;
        } else {
            self.reject_stake += vote.stake_weight;
        }

        self.votes.push(vote);
        Ok(())
    }

    /// Check if proposal has reached 2/3+ approval
    pub fn has_quorum(&self) -> bool {
        // Require 2/3+ stake weight to approve
        self.approve_stake * 3 > self.total_stake * 2
    }

    /// Get approval percentage
    pub fn approval_percentage(&self) -> f64 {
        if self.total_stake == 0 {
            return 0.0;
        }
        (self.approve_stake as f64 / self.total_stake as f64) * 100.0
    }

    /// Get the number of votes cast
    pub fn vote_count(&self) -> usize {
        self.votes.len()
    }
}

impl fmt::Display for VotingResults {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Voting {{ proposal: {}, approve: {}/{} ({:.1}%), quorum: {} }}",
            self.proposal_id,
            self.approve_stake,
            self.total_stake,
            self.approval_percentage(),
            self.has_quorum()
        )
    }
}

/// Approved upgrade ready for activation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovedUpgrade {
    /// The approved proposal
    pub proposal: UpgradeProposal,

    /// Voting results showing approval
    pub voting_results: VotingResults,

    /// Snapshot digest where approval was finalized
    pub approval_snapshot: SnapshotDigest,
}

impl ApprovedUpgrade {
    /// Create a new approved upgrade
    pub fn new(
        proposal: UpgradeProposal,
        voting_results: VotingResults,
        approval_snapshot: SnapshotDigest,
    ) -> Result<Self> {
        // Verify quorum
        if !voting_results.has_quorum() {
            return Err(Error::InvalidData(format!(
                "Proposal does not have quorum: {:.1}% approval",
                voting_results.approval_percentage()
            )));
        }

        Ok(Self {
            proposal,
            voting_results,
            approval_snapshot,
        })
    }
}
