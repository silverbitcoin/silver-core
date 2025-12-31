//! Comprehensive Integration Tests - Phase 4-6 Production-Grade
//!
//! Tests for:
//! - Database + RPC integration
//! - Cache + database integration
//! - Error scenarios
//! - Performance characteristics
//! - End-to-end workflows

use silver_core::cache_database_integration::{CacheDatabaseBackend, CachePolicy, QueryPath};
use silver_core::rpc_database_integration::{
    BlockData, RpcDatabaseBackend, TransactionData, UTXOData,
};

#[tokio::test]
async fn test_rpc_database_integration_block_queries() {
    let backend = RpcDatabaseBackend::new();

    // Create test block
    let block = BlockData {
        hash: "a".repeat(128),
        height: 100,
        version: 1,
        previous_hash: "b".repeat(128),
        merkle_root: "c".repeat(128),
        timestamp: 1234567890,
        bits: "1d00ffff".to_string(),
        nonce: 12345,
        tx_count: 1,
        size: 1000,
        confirmations: 10,
    };

    // Store block
    let store_result = backend.store_block(block.clone()).await;
    assert!(store_result.is_ok(), "Failed to store block");

    // Query by hash
    let query_result = backend.get_block_by_hash(&block.hash).await;
    assert!(query_result.is_ok(), "Failed to query block by hash");

    let result = query_result.unwrap();
    assert!(result.success, "Query should be successful");
    assert!(result.data.is_some(), "Query should return data");

    let retrieved_block = result.data.unwrap();
    assert_eq!(retrieved_block.hash, block.hash);
    assert_eq!(retrieved_block.height, block.height);

    // Query by height
    let height_query = backend.get_block_by_height(100).await;
    assert!(height_query.is_ok(), "Failed to query block by height");

    let height_result = height_query.unwrap();
    assert!(height_result.success, "Height query should be successful");
}

#[tokio::test]
async fn test_rpc_database_integration_transaction_queries() {
    let backend = RpcDatabaseBackend::new();

    // Create test transaction
    let tx = TransactionData {
        txid: "d".repeat(128),
        version: 1,
        locktime: 0,
        input_count: 1,
        output_count: 1,
        size: 250,
        fee: 1000,
        confirmations: 5,
        blockheight: Some(100),
        time: 1234567890,
        is_coinbase: false,
    };

    // Store transaction
    let store_result = backend.store_transaction(tx.clone()).await;
    assert!(store_result.is_ok(), "Failed to store transaction");

    // Query transaction
    let query_result = backend.get_transaction_by_txid(&tx.txid).await;
    assert!(query_result.is_ok(), "Failed to query transaction");

    let result = query_result.unwrap();
    assert!(result.success, "Query should be successful");
    assert!(result.data.is_some(), "Query should return data");

    let retrieved_tx = result.data.unwrap();
    assert_eq!(retrieved_tx.txid, tx.txid);
    assert_eq!(retrieved_tx.fee, tx.fee);
}

#[tokio::test]
async fn test_rpc_database_integration_utxo_queries() {
    let backend = RpcDatabaseBackend::new();

    // Create test UTXO
    let utxo = UTXOData {
        txid: "e".repeat(128),
        vout: 0,
        amount: 5000000000,
        script_pubkey: "76a914".to_string(),
        address: Some("1A1z7agoat".to_string()),
        confirmations: 10,
        spendable: true,
        solvable: true,
    };

    // Store UTXO
    let store_result = backend.store_utxo(utxo.clone()).await;
    assert!(store_result.is_ok(), "Failed to store UTXO");

    // Query UTXO
    let query_result = backend.get_utxo(&utxo.txid, utxo.vout).await;
    assert!(query_result.is_ok(), "Failed to query UTXO");

    let result = query_result.unwrap();
    assert!(result.success, "Query should be successful");
    assert!(result.data.is_some(), "Query should return data");

    let retrieved_utxo = result.data.unwrap();
    assert_eq!(retrieved_utxo.txid, utxo.txid);
    assert_eq!(retrieved_utxo.amount, utxo.amount);
}

#[tokio::test]
async fn test_rpc_database_cache_statistics() {
    let backend = RpcDatabaseBackend::new();

    // Create and store multiple blocks
    for i in 0..5 {
        let block = BlockData {
            hash: format!("{:0>128}", i),
            height: 100 + i as u64,
            version: 1,
            previous_hash: format!("{:0>128}", i - 1),
            merkle_root: "c".repeat(128),
            timestamp: 1234567890 + i as u64,
            bits: "1d00ffff".to_string(),
            nonce: 12345 + i as u64,
            tx_count: 1,
            size: 1000,
            confirmations: 10,
        };

        let _ = backend.store_block(block).await;
    }

    // Query blocks multiple times
    for i in 0..5 {
        let hash = format!("{:0>128}", i);
        let _ = backend.get_block_by_hash(&hash).await;
        let _ = backend.get_block_by_hash(&hash).await; // Second query for cache hit
    }

    // Check statistics
    let stats = backend.get_stats().await;
    assert!(stats.total_queries > 0, "Should have queries");
    assert!(stats.cache_hits > 0, "Should have cache hits");
    assert!(stats.cache_hit_rate > 0.0, "Should have positive hit rate");
}

#[tokio::test]
async fn test_cache_database_integration_basic() {
    let policy = CachePolicy::default();
    let backend = CacheDatabaseBackend::<String>::new(policy);

    // First query - cache miss
    let result1 = backend
        .get_or_fetch("key1", async { Ok("value1".to_string()) })
        .await;

    assert!(result1.is_ok());
    let query1 = result1.unwrap();
    assert_eq!(query1.path, QueryPath::CacheMissDatabase);

    // Second query - cache hit
    let result2 = backend
        .get_or_fetch("key1", async { Ok("value1".to_string()) })
        .await;

    assert!(result2.is_ok());
    let query2 = result2.unwrap();
    assert_eq!(query2.path, QueryPath::Cache);
}

#[tokio::test]
async fn test_cache_database_integration_invalidation() {
    let policy = CachePolicy::default();
    let backend = CacheDatabaseBackend::<String>::new(policy);

    // Store multiple entries
    for i in 0..10 {
        let key = format!("block:{}", i);
        let _ = backend
            .get_or_fetch(&key, async { Ok(format!("block_data_{}", i)) })
            .await;
    }

    // Verify entries are cached
    assert_eq!(backend.get_size().await, 10);

    // Invalidate pattern
    let removed = backend.invalidate_pattern("block:").await.unwrap();
    assert_eq!(removed, 10);
    assert_eq!(backend.get_size().await, 0);
}

#[tokio::test]
async fn test_cache_database_integration_statistics() {
    let policy = CachePolicy::default();
    let backend = CacheDatabaseBackend::<String>::new(policy);

    // Perform queries
    for i in 0..5 {
        let key = format!("key{}", i);
        let _ = backend
            .get_or_fetch(&key, async { Ok(format!("value{}", i)) })
            .await;
    }

    // Repeat queries for cache hits
    for i in 0..5 {
        let key = format!("key{}", i);
        let _ = backend
            .get_or_fetch(&key, async { Ok(format!("value{}", i)) })
            .await;
    }

    let stats = backend.get_stats().await;
    assert_eq!(stats.hits, 5, "Should have 5 cache hits");
    assert_eq!(stats.misses, 5, "Should have 5 cache misses");
    assert!(
        stats.hit_rate > 0.0 && stats.hit_rate <= 1.0,
        "Hit rate should be valid"
    );
}

#[tokio::test]
async fn test_error_handling_invalid_block_hash() {
    let backend = RpcDatabaseBackend::new();

    // Test empty hash
    let result = backend.get_block_by_hash("").await;
    assert!(result.is_err(), "Empty hash should fail");

    // Test invalid length
    let result = backend.get_block_by_hash("abc").await;
    assert!(result.is_err(), "Invalid length should fail");

    // Test invalid hex
    let result = backend.get_block_by_hash(&"z".repeat(128)).await;
    assert!(result.is_err(), "Invalid hex should fail");
}

#[tokio::test]
async fn test_error_handling_invalid_transaction_id() {
    let backend = RpcDatabaseBackend::new();

    // Test empty txid
    let result = backend.get_transaction_by_txid("").await;
    assert!(result.is_err(), "Empty txid should fail");

    // Test invalid length
    let result = backend.get_transaction_by_txid("abc").await;
    assert!(result.is_err(), "Invalid length should fail");

    // Test invalid hex
    let result = backend.get_transaction_by_txid(&"z".repeat(128)).await;
    assert!(result.is_err(), "Invalid hex should fail");
}

#[tokio::test]
async fn test_error_handling_cache_operations() {
    let policy = CachePolicy::default();
    let backend = CacheDatabaseBackend::<String>::new(policy);

    // Test invalid key
    let result = backend.invalidate("").await;
    assert!(result.is_err(), "Empty key should fail");

    // Test invalid pattern
    let result = backend.invalidate_pattern("").await;
    assert!(result.is_err(), "Empty pattern should fail");
}

#[tokio::test]
async fn test_performance_rpc_database_lookups() {
    let backend = RpcDatabaseBackend::new();

    // Store 100 blocks
    for i in 0..100 {
        let block = BlockData {
            hash: format!("{:0>128}", i),
            height: 1000 + i as u64,
            version: 1,
            previous_hash: format!("{:0>128}", i - 1),
            merkle_root: "c".repeat(128),
            timestamp: 1234567890 + i as u64,
            bits: "1d00ffff".to_string(),
            nonce: 12345 + i as u64,
            tx_count: 1,
            size: 1000,
            confirmations: 10,
        };

        let _ = backend.store_block(block).await;
    }

    // Measure query performance
    let start = std::time::Instant::now();

    for i in 0..100 {
        let hash = format!("{:0>128}", i);
        let _ = backend.get_block_by_hash(&hash).await;
    }

    let elapsed = start.elapsed();

    // All queries should be fast (cache hits)
    let avg_time = elapsed.as_micros() as f64 / 100.0;
    assert!(avg_time < 1000.0, "Average query time should be < 1ms");
}

#[tokio::test]
async fn test_performance_cache_database_lookups() {
    let policy = CachePolicy::default();
    let backend = CacheDatabaseBackend::<String>::new(policy);

    // Populate cache
    for i in 0..100 {
        let key = format!("key{}", i);
        let _ = backend
            .get_or_fetch(&key, async { Ok(format!("value{}", i)) })
            .await;
    }

    // Measure cache hit performance
    let start = std::time::Instant::now();

    for i in 0..100 {
        let key = format!("key{}", i);
        let _ = backend
            .get_or_fetch(&key, async { Ok(format!("value{}", i)) })
            .await;
    }

    let elapsed = start.elapsed();

    // All queries should be very fast (cache hits)
    let avg_time = elapsed.as_micros() as f64 / 100.0;
    assert!(avg_time < 500.0, "Average cache hit time should be < 0.5ms");
}

#[tokio::test]
async fn test_end_to_end_block_workflow() {
    let rpc_backend = RpcDatabaseBackend::new();
    let cache_policy = CachePolicy::default();
    let cache_backend = CacheDatabaseBackend::<BlockData>::new(cache_policy);

    // Create block
    let block = BlockData {
        hash: "f".repeat(128),
        height: 500,
        version: 1,
        previous_hash: "g".repeat(128),
        merkle_root: "h".repeat(128),
        timestamp: 1234567890,
        bits: "1d00ffff".to_string(),
        nonce: 54321,
        tx_count: 2,
        size: 2000,
        confirmations: 20,
    };

    // Store in RPC backend
    let _ = rpc_backend.store_block(block.clone()).await;

    // Query through cache backend
    let cache_result = cache_backend
        .get_or_fetch(&block.hash, async {
            // Simulate database fetch
            Ok(block.clone())
        })
        .await;

    assert!(cache_result.is_ok());
    let result = cache_result.unwrap();
    assert_eq!(result.path, QueryPath::CacheMissDatabase);

    // Second query should be cache hit
    let cache_result2 = cache_backend
        .get_or_fetch(&block.hash, async {
            // This shouldn't be called
            panic!("Should use cache");
        })
        .await;

    assert!(cache_result2.is_ok());
    let result2 = cache_result2.unwrap();
    assert_eq!(result2.path, QueryPath::Cache);
}

#[tokio::test]
async fn test_end_to_end_transaction_workflow() {
    let rpc_backend = RpcDatabaseBackend::new();
    let cache_policy = CachePolicy::default();
    let cache_backend = CacheDatabaseBackend::<TransactionData>::new(cache_policy);

    // Create transaction
    let tx = TransactionData {
        txid: "e".repeat(128),
        version: 1,
        locktime: 0,
        input_count: 2,
        output_count: 2,
        size: 500,
        fee: 2000,
        confirmations: 3,
        blockheight: Some(500),
        time: 1234567890,
        is_coinbase: false,
    };

    // Store in RPC backend
    let store_result = rpc_backend.store_transaction(tx.clone()).await;
    assert!(store_result.is_ok(), "Failed to store transaction");

    // Query through cache backend
    let cache_result = cache_backend
        .get_or_fetch(&tx.txid, async { Ok(tx.clone()) })
        .await;

    assert!(cache_result.is_ok());
    let result = cache_result.unwrap();
    assert_eq!(result.path, QueryPath::CacheMissDatabase);

    // Verify RPC backend has the transaction
    let rpc_result = rpc_backend.get_transaction_by_txid(&tx.txid).await;
    assert!(
        rpc_result.is_ok(),
        "Transaction should be found in RPC backend"
    );
}

#[tokio::test]
async fn test_concurrent_cache_operations() {
    let policy = CachePolicy::default();
    let backend = std::sync::Arc::new(CacheDatabaseBackend::<String>::new(policy));

    let mut handles = vec![];

    // Spawn 10 concurrent tasks
    for i in 0..10 {
        let backend_clone = backend.clone();
        let handle = tokio::spawn(async move {
            for j in 0..10 {
                let key = format!("key_{}_{}", i, j);
                let _ = backend_clone
                    .get_or_fetch(&key, async { Ok(format!("value_{}_{}", i, j)) })
                    .await;
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        let _ = handle.await;
    }

    // Verify cache has all entries
    let size = backend.get_size().await;
    assert_eq!(size, 100, "Cache should have 100 entries");
}

#[tokio::test]
async fn test_cache_cleanup_expired_entries() {
    let mut policy = CachePolicy::default();
    policy.default_ttl_seconds = 1; // 1 second TTL

    let backend = CacheDatabaseBackend::<String>::new(policy);

    // Store entry
    let _ = backend
        .get_or_fetch("key1", async { Ok("value1".to_string()) })
        .await;

    assert_eq!(backend.get_size().await, 1);

    // Wait for expiration
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Cleanup
    let removed = backend.cleanup_expired().await.unwrap();
    assert_eq!(removed, 1);
    assert_eq!(backend.get_size().await, 0);
}

#[tokio::test]
async fn test_rpc_database_cache_sizes() {
    let backend = RpcDatabaseBackend::new();

    // Store various data types
    for i in 0..5 {
        let block = BlockData {
            hash: format!("a{:0>127}", i),
            height: 100 + i as u64,
            version: 1,
            previous_hash: "b".repeat(128),
            merkle_root: "c".repeat(128),
            timestamp: 1234567890,
            bits: "1d00ffff".to_string(),
            nonce: 12345,
            tx_count: 1,
            size: 1000,
            confirmations: 10,
        };

        let _ = backend.store_block(block).await;
    }

    for i in 0..3 {
        let tx = TransactionData {
            txid: format!("d{:0>127}", i),
            version: 1,
            locktime: 0,
            input_count: 1,
            output_count: 1,
            size: 250,
            fee: 1000,
            confirmations: 5,
            blockheight: Some(100),
            time: 1234567890,
            is_coinbase: false,
        };

        let _ = backend.store_transaction(tx).await;
    }

    let sizes = backend.get_cache_sizes().await.unwrap();
    assert_eq!(sizes["blocks"].as_u64().unwrap(), 5);
    assert_eq!(sizes["transactions"].as_u64().unwrap(), 3);
}
