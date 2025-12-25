//! Data models for persistent storage in ParityDB
//! All structures are Bitcoin-compatible and serializable
//! Matches Bitcoin Core's data storage strategy exactly

use serde::{Deserialize, Serialize};

/// Blockchain info stored in ParityDB
/// Contains overall blockchain state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainInfo {
    /// Chain name: "mainnet", "testnet", "signet"
    pub chain: String,
    /// Current block count
    pub blocks: u64,
    /// Header count
    pub headers: u64,
    /// Best block hash (128 hex chars for 512-bit)
    pub bestblockhash: String,
    /// Current difficulty
    pub difficulty: f64,
    /// Median block time (UNIX epoch)
    pub mediantime: u64,
    /// Verification progress (0.0 to 1.0)
    pub verificationprogress: f64,
    /// Initial block download flag
    pub initialblockdownload: bool,
    /// Chain work (cumulative difficulty)
    pub chainwork: String,
    /// Size on disk in bytes
    pub size_on_disk: u64,
    /// Pruned flag
    pub pruned: bool,
    /// Warning messages
    pub warnings: String,
}

/// Block data stored in ParityDB
/// Contains all information needed to reconstruct block details
/// Matches Bitcoin Core's getblock response exactly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockData {
    /// Block hash (128 hex chars for 512-bit)
    pub hash: String,
    /// Number of confirmations
    pub confirmations: u64,
    /// Block size in bytes
    pub size: u64,
    /// Block size without witness data
    pub strippedsize: u64,
    /// Block weight (BIP 141)
    pub weight: u64,
    /// Block height
    pub height: u64,
    /// Block version
    pub version: u32,
    /// Block version in hex
    pub versionhex: String,
    /// Merkle root hash (128 hex chars)
    pub merkleroot: String,
    /// Transaction IDs in this block (128 hex chars each)
    pub tx: Vec<String>,
    /// Block timestamp (UNIX epoch)
    pub time: u64,
    /// Median block time (UNIX epoch)
    pub mediantime: u64,
    /// Nonce value
    pub nonce: u64,
    /// Difficulty bits
    pub bits: String,
    /// Difficulty value
    pub difficulty: f64,
    /// Chain work
    pub chainwork: String,
    /// Number of transactions
    pub ntx: u64,
    /// Previous block hash (128 hex chars)
    pub previousblockhash: String,
    /// Next block hash (128 hex chars) - optional
    pub nextblockhash: Option<String>,
    /// Miner address (SLVR + base58)
    pub miner: String,
    /// Block reward in MIST
    pub reward: u128,
}

/// Transaction data stored in ParityDB
/// Contains all information needed to reconstruct transaction details
/// Matches Bitcoin Core's gettransaction response exactly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionData {
    /// Transaction ID (128 hex chars for 512-bit)
    pub txid: String,
    /// Transaction hash (same as txid for 512-bit)
    pub hash: String,
    /// Transaction version
    pub version: u32,
    /// Transaction size in bytes
    pub size: u64,
    /// Virtual size (for witness)
    pub vsize: u64,
    /// Transaction weight (BIP 141)
    pub weight: u64,
    /// Lock time
    pub locktime: u32,
    /// Transaction inputs
    pub vin: Vec<TransactionInput>,
    /// Transaction outputs
    pub vout: Vec<TransactionOutput>,
    /// Transaction amount in MIST
    pub amount: u128,
    /// Transaction fee in MIST
    pub fee: u128,
    /// Number of confirmations
    pub confirmations: u64,
    /// Whether this is a coinbase transaction
    pub generated: bool,
    /// Whether transaction is trusted
    pub trusted: bool,
    /// Block hash containing this transaction (128 hex chars)
    pub blockhash: String,
    /// Block height containing this transaction
    pub blockheight: u64,
    /// Index in block
    pub blockindex: u64,
    /// Block time (UNIX epoch)
    pub blocktime: u64,
    /// Transaction time (UNIX epoch)
    pub time: u64,
    /// Time received (UNIX epoch)
    pub timereceived: u64,
    /// Associated comment
    pub comment: Option<String>,
    /// BIP 125 replaceable: "yes", "no", or "unknown"
    pub bip125_replaceable: String,
    /// Transaction details (inputs/outputs)
    pub details: Vec<TransactionDetail>,
    /// Raw transaction hex
    pub hex: String,
}

/// Transaction input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInput {
    /// Previous transaction ID (128 hex chars)
    pub txid: String,
    /// Previous output index
    pub vout: u64,
    /// Script signature
    pub scriptsig: ScriptSig,
    /// Sequence number
    pub sequence: u32,
    /// Coinbase data (for coinbase transactions)
    pub coinbase: Option<String>,
}

/// Transaction output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionOutput {
    /// Output value in MIST
    pub value: u128,
    /// Output index
    pub n: u64,
    /// Script public key
    pub scriptpubkey: ScriptPubKey,
}

/// Script signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptSig {
    /// Assembly representation
    pub asm: String,
    /// Hex representation
    pub hex: String,
}

/// Script public key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptPubKey {
    /// Assembly representation
    pub asm: String,
    /// Hex representation
    pub hex: String,
    /// Required signatures
    pub reqsigs: u64,
    /// Script type: "pubkeyhash", "pubkey", "multisig", "scripthash"
    pub type_: String,
    /// Addresses involved
    pub addresses: Vec<String>,
}

/// Transaction detail (input/output)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetail {
    /// Involves watch-only address
    #[serde(rename = "involvesWatchonly")]
    pub involves_watchonly: bool,
    /// Address involved in transaction
    pub address: String,
    /// Category: "send", "receive", or "generate"
    pub category: String,
    /// Amount in MIST
    pub amount: u128,
    /// Label
    pub label: Option<String>,
    /// Output index
    pub vout: u64,
    /// Fee in MIST
    pub fee: Option<u128>,
    /// Abandoned flag
    pub abandoned: bool,
}

/// Address data stored in ParityDB
/// Contains all information about an address
/// Matches Bitcoin Core's getaddressinfo response exactly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressData {
    /// Address (SLVR + base58, 90-92 chars)
    pub address: String,
    /// Script public key (hex)
    pub scriptpubkey: String,
    /// Is this address ours
    pub ismine: bool,
    /// Is watch-only
    pub iswatchonly: bool,
    /// Is solvable
    pub solvable: bool,
    /// Descriptor
    pub desc: Option<String>,
    /// Is script
    pub isscript: bool,
    /// Is change address
    pub ischange: bool,
    /// Is witness address
    pub iswitness: bool,
    /// Witness version
    pub witness_version: Option<u32>,
    /// Witness program (hex)
    pub witness_program: Option<String>,
    /// Script type
    pub script: Option<String>,
    /// Redeemscript (hex)
    pub hex: Option<String>,
    /// Public keys
    pub pubkeys: Vec<String>,
    /// Signatures required
    pub sigsrequired: Option<u64>,
    /// Public key (hex)
    pub pubkey: Option<String>,
    /// Is compressed
    pub iscompressed: Option<bool>,
    /// Creation time (UNIX epoch)
    pub timestamp: u64,
    /// HD key path
    pub hdkeypath: Option<String>,
    /// HD seed ID
    pub hdseedid: Option<String>,
    /// HD master fingerprint
    pub hdmasterfingerprint: Option<String>,
    /// Address labels
    pub labels: Vec<String>,
    /// Total balance in MIST
    pub balance: u128,
    /// Number of blocks mined by this address
    pub blocks_mined: u64,
    /// Total reward earned in MIST
    pub total_reward: u128,
    /// First block time (UNIX epoch)
    pub first_block_time: u64,
    /// Last block time (UNIX epoch)
    pub last_block_time: u64,
}

/// Account balance stored in ParityDB
/// Contains balance information for an account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    /// Address
    pub address: String,
    /// Balance in MIST
    pub balance_mist: u128,
    /// Balance in SLVR
    pub balance_slvr: f64,
    /// Confirmed balance in MIST
    pub confirmed: u128,
    /// Unconfirmed balance in MIST
    pub unconfirmed: u128,
    /// Total balance in MIST
    pub total: u128,
    /// Minimum confirmations
    pub minconf: u64,
    /// Include watch-only
    pub include_watchonly: bool,
    /// Avoid reuse
    pub avoid_reuse: bool,
    /// Last updated (UNIX epoch)
    pub last_updated: u64,
}

/// Mining info stored in ParityDB
/// Contains current mining state and statistics
/// Matches Bitcoin Core's getmininginfo response exactly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningInfo {
    /// Current block count
    pub blocks: u64,
    /// Current block size
    pub currentblocksize: u64,
    /// Current block transaction count
    pub currentblocktx: u64,
    /// Current difficulty
    pub difficulty: f64,
    /// Error messages
    pub errors: String,
    /// Mining enabled flag
    pub generate: bool,
    /// Mining thread count
    pub genproclimit: u64,
    /// Hashes per second
    pub hashespersec: f64,
    /// Mining reward address (SLVR)
    pub miningaddress: String,
    /// Miner's balance in SLVR
    pub minerbalance: f64,
    /// Network hashrate
    pub networkhashps: f64,
    /// Pooled transactions
    pub pooledtx: u64,
    /// Testnet flag
    pub testnet: bool,
    /// Last block time (UNIX epoch)
    pub last_block_time: u64,
    /// Blocks found
    pub blocks_found: u64,
    /// Total reward in MIST
    pub total_reward: u128,
}

/// Network info stored in ParityDB
/// Contains network state and statistics
/// Matches Bitcoin Core's getnetworkinfo response exactly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// Node version
    pub version: u32,
    /// Subversion string
    pub subversion: String,
    /// Protocol version
    pub protocolversion: u32,
    /// Local services
    pub localservices: String,
    /// Local services names
    pub localservicesnames: Vec<String>,
    /// Time offset
    pub timeoffset: i64,
    /// Network active flag
    pub networkactive: bool,
    /// Connection count
    pub connections: u64,
    /// Inbound connections
    pub connections_in: u64,
    /// Outbound connections
    pub connections_out: u64,
    /// Network details
    pub networks: Vec<NetworkDetail>,
    /// Relay fee
    pub relayfee: f64,
    /// Incremental fee
    pub incrementalfee: f64,
    /// Local addresses
    pub localaddresses: Vec<LocalAddress>,
    /// Warnings
    pub warnings: String,
}

/// Network detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDetail {
    /// Network name
    pub name: String,
    /// Limited flag
    pub limited: bool,
    /// Reachable flag
    pub reachable: bool,
    /// Proxy
    pub proxy: String,
    /// Proxy randomize port
    pub proxy_randomize_port: bool,
}

/// Local address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAddress {
    /// Address
    pub address: String,
    /// Port
    pub port: u16,
    /// Score
    pub score: u64,
}

/// Wallet info stored in ParityDB
/// Contains wallet state and statistics
/// Matches Bitcoin Core's getwalletinfo response exactly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    /// Wallet name
    pub walletname: String,
    /// Wallet version
    pub walletversion: u32,
    /// Total balance in SLVR
    pub balance: f64,
    /// Total balance in MIST
    pub balance_mist: u128,
    /// Unconfirmed balance in SLVR
    pub unconfirmed_balance: f64,
    /// Immature balance in SLVR
    pub immature_balance: f64,
    /// Transaction count
    pub txcount: u64,
    /// Key pool oldest (UNIX epoch)
    pub keypoololdest: u64,
    /// Key pool size
    pub keypoolsize: u64,
    /// HD internal key pool size
    pub keypoolsize_hd_internal: u64,
    /// Pay transaction fee
    pub paytxfee: f64,
    /// Private keys enabled flag
    pub private_keys_enabled: bool,
    /// Avoid reuse flag
    pub avoid_reuse: bool,
    /// Scanning flag
    pub scanning: bool,
    /// Descriptors enabled flag
    pub descriptors: bool,
    /// Total addresses
    pub total_addresses: u64,
    /// Total blocks mined
    pub total_blocks_mined: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_info_serialization() {
        let info = BlockchainInfo {
            chain: "mainnet".to_string(),
            blocks: 850000,
            headers: 850000,
            bestblockhash: "0".repeat(128),
            difficulty: 84000000000.0,
            mediantime: 1700000000,
            verificationprogress: 0.9999,
            initialblockdownload: false,
            chainwork: "0".repeat(64),
            size_on_disk: 500000000,
            pruned: false,
            warnings: String::new(),
        };

        let json = serde_json::to_string(&info).unwrap();
        let deserialized: BlockchainInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(info.blocks, deserialized.blocks);
    }

    #[test]
    fn test_block_data_serialization() {
        let block = BlockData {
            hash: "0".repeat(128),
            confirmations: 100,
            size: 1024,
            strippedsize: 1000,
            weight: 4000,
            height: 1,
            version: 1,
            versionhex: "01000000".to_string(),
            merkleroot: "0".repeat(128),
            tx: vec!["0".repeat(128)],
            time: 1234567890,
            mediantime: 1234567890,
            nonce: 12345,
            bits: "207fffff".to_string(),
            difficulty: 1.0,
            chainwork: "0".repeat(64),
            ntx: 1,
            previousblockhash: "0".repeat(128),
            nextblockhash: None,
            miner: "SLVRtest".to_string(),
            reward: 50_000_000_000,
        };

        let json = serde_json::to_string(&block).unwrap();
        let deserialized: BlockData = serde_json::from_str(&json).unwrap();
        assert_eq!(block.height, deserialized.height);
        assert_eq!(block.miner, deserialized.miner);
    }

    #[test]
    fn test_transaction_data_serialization() {
        let tx = TransactionData {
            txid: "0".repeat(128),
            hash: "0".repeat(128),
            version: 1,
            size: 225,
            vsize: 225,
            weight: 900,
            locktime: 0,
            vin: vec![],
            vout: vec![],
            amount: 100_000_000,
            fee: 1_000_000,
            confirmations: 10,
            generated: true,
            trusted: true,
            blockhash: "0".repeat(128),
            blockheight: 1,
            blockindex: 0,
            blocktime: 1234567890,
            time: 1234567890,
            timereceived: 1234567890,
            comment: None,
            bip125_replaceable: "no".to_string(),
            details: vec![],
            hex: "0100000001".to_string(),
        };

        let json = serde_json::to_string(&tx).unwrap();
        let deserialized: TransactionData = serde_json::from_str(&json).unwrap();
        assert_eq!(tx.txid, deserialized.txid);
        assert_eq!(tx.generated, deserialized.generated);
    }

    #[test]
    fn test_address_data_serialization() {
        let addr = AddressData {
            address: "SLVRtest".to_string(),
            scriptpubkey: "76a914".to_string(),
            ismine: true,
            iswatchonly: false,
            solvable: true,
            desc: None,
            isscript: false,
            ischange: false,
            iswitness: false,
            witness_version: None,
            witness_program: None,
            script: None,
            hex: None,
            pubkeys: vec![],
            sigsrequired: None,
            pubkey: None,
            iscompressed: None,
            timestamp: 1234567890,
            hdkeypath: None,
            hdseedid: None,
            hdmasterfingerprint: None,
            labels: vec![],
            balance: 50_000_000_000,
            blocks_mined: 100,
            total_reward: 5_000_000_000_000,
            first_block_time: 1234567890,
            last_block_time: 1234567890,
        };

        let json = serde_json::to_string(&addr).unwrap();
        let deserialized: AddressData = serde_json::from_str(&json).unwrap();
        assert_eq!(addr.address, deserialized.address);
        assert_eq!(addr.blocks_mined, deserialized.blocks_mined);
    }

    #[test]
    fn test_account_balance_serialization() {
        let balance = AccountBalance {
            address: "SLVRtest".to_string(),
            balance_mist: 50_000_000_000,
            balance_slvr: 50.0,
            confirmed: 50_000_000_000,
            unconfirmed: 0,
            total: 50_000_000_000,
            minconf: 1,
            include_watchonly: false,
            avoid_reuse: false,
            last_updated: 1234567890,
        };

        let json = serde_json::to_string(&balance).unwrap();
        let deserialized: AccountBalance = serde_json::from_str(&json).unwrap();
        assert_eq!(balance.balance_mist, deserialized.balance_mist);
    }

    #[test]
    fn test_mining_info_serialization() {
        let mining = MiningInfo {
            blocks: 850000,
            currentblocksize: 1024,
            currentblocktx: 2500,
            difficulty: 84000000000.0,
            errors: String::new(),
            generate: true,
            genproclimit: 4,
            hashespersec: 1000000000.0,
            miningaddress: "SLVRtest".to_string(),
            minerbalance: 50.0,
            networkhashps: 500000000000.0,
            pooledtx: 100,
            testnet: false,
            last_block_time: 1234567890,
            blocks_found: 100,
            total_reward: 5_000_000_000_000,
        };

        let json = serde_json::to_string(&mining).unwrap();
        let deserialized: MiningInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(mining.blocks, deserialized.blocks);
        assert_eq!(mining.blocks_found, deserialized.blocks_found);
    }
}
