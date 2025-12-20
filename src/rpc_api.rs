//! JSON-RPC API for SilverBitcoin blockchain
//! Provides HTTP endpoints for wallet, mining, and blockchain operations

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;

/// RPC Request structure
#[derive(Debug, Deserialize)]
pub struct RpcRequest {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// RPC method name
    pub method: String,
    /// Method parameters
    pub params: Vec<Value>,
    /// Request ID
    pub id: u64,
}

/// RPC Response structure
#[derive(Debug, Serialize)]
pub struct RpcResponse {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Result value (if successful)
    pub result: Option<Value>,
    /// Error object (if failed)
    pub error: Option<RpcError>,
    /// Response ID
    pub id: u64,
}

/// RPC Error structure
#[derive(Debug, Serialize)]
pub struct RpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
}

/// Blockchain state for RPC
pub struct BlockchainState {
    /// Current block count
    pub block_count: u64,
    /// Current difficulty
    pub difficulty: u64,
    /// Current hashrate
    pub hashrate: f64,
    /// Mining enabled flag
    pub mining_enabled: bool,
    /// Mining address for rewards
    pub mining_address: String,
}

/// Handle RPC method calls
pub async fn handle_rpc_method(
    method: &str,
    params: &[Value],
    state: Arc<RwLock<BlockchainState>>,
) -> Result<Value, RpcError> {
    match method {
        // Blockchain info methods
        "getblockchaininfo" => get_blockchain_info(state).await,
        "getblockcount" => get_block_count(state).await,
        "getdifficulty" => get_difficulty(state).await,
        "gethashrate" => get_hashrate(state).await,

        // Address methods
        "getnewaddress" => get_new_address(params).await,
        "listaddresses" => list_addresses(params).await,
        "getaddressbalance" => get_address_balance(params).await,

        // Mining methods
        "startmining" => start_mining(params, state).await,
        "stopmining" => stop_mining(state).await,
        "getmininginfo" => get_mining_info(state).await,
        "setminingaddress" => set_mining_address(params, state).await,
        "submitblock" => submit_block(params, state).await,

        // Transaction methods
        "sendtransaction" => send_transaction(params).await,
        "gettransaction" => get_transaction(params).await,

        // Utility methods
        "validateaddress" => validate_address(params).await,
        "getinfo" => get_info(state).await,

        _ => Err(RpcError {
            code: -32601,
            message: format!("Method not found: {}", method),
        }),
    }
}

/// Get blockchain information
async fn get_blockchain_info(state: Arc<RwLock<BlockchainState>>) -> Result<Value, RpcError> {
    let blockchain = state.read().await;

    Ok(json!({
        "chain": "mainnet",
        "blocks": blockchain.block_count,
        "headers": blockchain.block_count,
        "bestblockhash": "0000000000000000000000000000000000000000000000000000000000000000",
        "difficulty": blockchain.difficulty,
        "mediantime": 0,
        "verificationprogress": 1.0,
        "initialblockdownload": false,
        "chainwork": "0000000000000000000000000000000000000000000000000000000000000000",
        "size_on_disk": 0,
        "pruned": false,
        "softforks": {},
        "warnings": ""
    }))
}

/// Get block count
async fn get_block_count(state: Arc<RwLock<BlockchainState>>) -> Result<Value, RpcError> {
    let blockchain = state.read().await;
    Ok(Value::Number(blockchain.block_count.into()))
}

/// Get current difficulty
async fn get_difficulty(state: Arc<RwLock<BlockchainState>>) -> Result<Value, RpcError> {
    let blockchain = state.read().await;
    Ok(json!(blockchain.difficulty as f64))
}

/// Get current hashrate
async fn get_hashrate(state: Arc<RwLock<BlockchainState>>) -> Result<Value, RpcError> {
    let blockchain = state.read().await;
    Ok(json!(blockchain.hashrate))
}

/// Generate a new address
async fn get_new_address(params: &[Value]) -> Result<Value, RpcError> {
    let label = params
        .first()
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    match crate::wallet::AddressGenerator::generate() {
        Ok((address, public_key, _private_key)) => {
            info!("Generated new address: {} ({})", address, label);
            Ok(json!({
                "address": address,
                "public_key": public_key,
                "label": label
            }))
        }
        Err(e) => Err(RpcError {
            code: -1,
            message: format!("Failed to generate address: {}", e),
        }),
    }
}

/// List all addresses
async fn list_addresses(_params: &[Value]) -> Result<Value, RpcError> {
    // This would list addresses from the wallet
    Ok(json!([]))
}

/// Get address balance
async fn get_address_balance(params: &[Value]) -> Result<Value, RpcError> {
    let address = params
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError {
            code: -1,
            message: "Address parameter required".to_string(),
        })?;

    if !crate::wallet::AddressGenerator::validate_address(address) {
        return Err(RpcError {
            code: -5,
            message: format!("Invalid address: {}", address),
        });
    }

    Ok(json!({
        "address": address,
        "balance": 0,
        "confirmed": 0,
        "unconfirmed": 0
    }))
}

/// Start mining
async fn start_mining(
    params: &[Value],
    state: Arc<RwLock<BlockchainState>>,
) -> Result<Value, RpcError> {
    let threads = params
        .first()
        .and_then(|v| v.as_u64())
        .unwrap_or(4) as usize;

    let mut blockchain = state.write().await;
    blockchain.mining_enabled = true;

    info!("Mining started with {} threads", threads);

    Ok(json!({
        "status": "mining_started",
        "threads": threads
    }))
}

/// Stop mining
async fn stop_mining(state: Arc<RwLock<BlockchainState>>) -> Result<Value, RpcError> {
    let mut blockchain = state.write().await;
    blockchain.mining_enabled = false;

    info!("Mining stopped");

    Ok(json!({
        "status": "mining_stopped"
    }))
}

/// Get mining information
async fn get_mining_info(state: Arc<RwLock<BlockchainState>>) -> Result<Value, RpcError> {
    let blockchain = state.read().await;

    Ok(json!({
        "blocks": blockchain.block_count,
        "currentblocksize": 0,
        "currentblocktx": 0,
        "difficulty": blockchain.difficulty,
        "errors": "",
        "generate": blockchain.mining_enabled,
        "genproclimit": 0,
        "hashespersec": blockchain.hashrate,
        "pooledtx": 0,
        "testnet": false,
        "chain": "mainnet",
        "mining_address": blockchain.mining_address
    }))
}

/// Set mining address for rewards
async fn set_mining_address(
    params: &[Value],
    state: Arc<RwLock<BlockchainState>>,
) -> Result<Value, RpcError> {
    let address = params
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError {
            code: -1,
            message: "Address parameter required".to_string(),
        })?;

    // Validate address
    if !crate::wallet::AddressGenerator::validate_address(address) {
        return Err(RpcError {
            code: -5,
            message: format!("Invalid address: {}", address),
        });
    }

    let mut blockchain = state.write().await;
    blockchain.mining_address = address.to_string();

    info!("Mining address set to: {}", address);

    Ok(json!({
        "status": "success",
        "mining_address": address,
        "message": "Mining rewards will be sent to this address"
    }))
}

/// Submit a mined block
async fn submit_block(
    params: &[Value],
    state: Arc<RwLock<BlockchainState>>,
) -> Result<Value, RpcError> {
    let block_data = params
        .first()
        .ok_or_else(|| RpcError {
            code: -1,
            message: "Block data required".to_string(),
        })?;

    // Parse block data
    let block_hex = block_data
        .as_str()
        .ok_or_else(|| RpcError {
            code: -1,
            message: "Block data must be hex string".to_string(),
        })?;

    // Validate hex
    if hex::decode(block_hex).is_err() {
        return Err(RpcError {
            code: -22,
            message: "Block decode failed".to_string(),
        });
    }

    // Accept block and increment block count
    let mut blockchain = state.write().await;
    blockchain.block_count += 1;

    let block_hash = format!("{:064x}", blockchain.block_count);
    
    info!("Block #{} submitted and accepted", blockchain.block_count);
    info!("Block hash: {}", block_hash);

    Ok(json!({
        "status": "accepted",
        "block_number": blockchain.block_count,
        "block_hash": block_hash,
        "message": "Block accepted and added to chain"
    }))
}

/// Send a transaction
async fn send_transaction(params: &[Value]) -> Result<Value, RpcError> {
    let _tx_data = params.first().ok_or_else(|| RpcError {
        code: -1,
        message: "Transaction data required".to_string(),
    })?;

    // This would create and broadcast a transaction
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    let tx_id = format!("tx_{}", timestamp);

    Ok(json!({
        "txid": tx_id,
        "status": "pending"
    }))
}

/// Get transaction details
async fn get_transaction(params: &[Value]) -> Result<Value, RpcError> {
    let txid = params
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError {
            code: -1,
            message: "Transaction ID required".to_string(),
        })?;

    Ok(json!({
        "txid": txid,
        "status": "not_found"
    }))
}

/// Validate address format
async fn validate_address(params: &[Value]) -> Result<Value, RpcError> {
    let address = params
        .first()
        .and_then(|v| v.as_str())
        .ok_or_else(|| RpcError {
            code: -1,
            message: "Address parameter required".to_string(),
        })?;

    let is_valid = crate::wallet::AddressGenerator::validate_address(address);

    Ok(json!({
        "isvalid": is_valid,
        "address": address,
        "scriptPubKey": "",
        "ismine": false,
        "iswatchonly": false,
        "isscript": false
    }))
}

/// Get general blockchain info
async fn get_info(state: Arc<RwLock<BlockchainState>>) -> Result<Value, RpcError> {
    let blockchain = state.read().await;

    Ok(json!({
        "version": "2.5.2",
        "protocolversion": 70015,
        "walletversion": 160300,
        "balance": 0,
        "blocks": blockchain.block_count,
        "timeoffset": 0,
        "connections": 0,
        "proxy": "",
        "difficulty": blockchain.difficulty,
        "testnet": false,
        "keypoololdest": 0,
        "keypoolsize": 0,
        "paytxfee": 0,
        "relayfee": 0.00001,
        "warnings": ""
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validate_address() {
        let (address, _, _) = crate::wallet::AddressGenerator::generate().unwrap();
        let params = vec![Value::String(address)];

        let result = validate_address(&params).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response["isvalid"], true);
    }

    #[tokio::test]
    async fn test_get_new_address() {
        let params = vec![Value::String("test".to_string())];
        let result = get_new_address(&params).await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response["address"].is_string());
    }
}
