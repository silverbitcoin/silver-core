//! Cache Layer - Phase 6 Performance Optimization
//!
//! Implements LRU caching for blocks, transactions, and UTXOs
//! to improve query performance and reduce database load.
//!
//! # Features
//! - LRU cache for blocks
//! - LRU cache for transactions
//! - LRU cache for UTXOs
//! - Cache invalidation
//! - Cache statistics
//! - Error handling

use crate::error::{Error, Result};
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;
use std::sync::{Arc, Mutex};
use tracing::debug;

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    /// Number of hits
    pub hits: u64,
    /// Number of misses
    pub misses: u64,
    /// Hit rate (0.0 to 1.0)
    pub hit_rate: f64,
    /// Current size
    pub current_size: usize,
    /// Maximum size
    pub max_size: usize,
}

/// Block cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedBlock {
    /// Block hash
    pub hash: String,
    /// Block height
    pub height: u64,
    /// Block data (serialized)
    pub data: Vec<u8>,
    /// Timestamp when cached
    pub cached_at: u64,
}

/// Transaction cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedTransaction {
    /// Transaction ID
    pub txid: String,
    /// Transaction data (serialized)
    pub data: Vec<u8>,
    /// Timestamp when cached
    pub cached_at: u64,
}

/// UTXO cache entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedUTXO {
    /// Transaction ID
    pub txid: String,
    /// Output index
    pub vout: u32,
    /// UTXO data (serialized)
    pub data: Vec<u8>,
    /// Timestamp when cached
    pub cached_at: u64,
}

/// Block cache with LRU eviction
pub struct BlockCache {
    cache: Arc<Mutex<LruCache<String, CachedBlock>>>,
    hits: Arc<Mutex<u64>>,
    misses: Arc<Mutex<u64>>,
}

impl BlockCache {
    /// Create a new block cache
    pub fn new(capacity: usize) -> Result<Self> {
        let cache_size = NonZeroUsize::new(capacity)
            .ok_or_else(|| Error::InvalidData("Cache capacity must be positive".to_string()))?;

        Ok(Self {
            cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        })
    }

    /// Get block from cache
    pub fn get(&self, hash: &str) -> Result<Option<CachedBlock>> {
        debug!("Getting block from cache: {}", hash);

        let mut cache = self
            .cache
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire cache lock: {}", e)))?;

        if let Some(block) = cache.get(hash) {
            let mut hits = self
                .hits
                .lock()
                .map_err(|e| Error::Internal(format!("Failed to acquire hits lock: {}", e)))?;
            *hits += 1;
            Ok(Some(block.clone()))
        } else {
            let mut misses = self
                .misses
                .lock()
                .map_err(|e| Error::Internal(format!("Failed to acquire misses lock: {}", e)))?;
            *misses += 1;
            Ok(None)
        }
    }

    /// Put block in cache
    pub fn put(&self, hash: String, block: CachedBlock) -> Result<()> {
        debug!("Putting block in cache: {}", hash);

        let mut cache = self
            .cache
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire cache lock: {}", e)))?;

        cache.put(hash, block);
        Ok(())
    }

    /// Remove block from cache
    pub fn remove(&self, hash: &str) -> Result<()> {
        debug!("Removing block from cache: {}", hash);

        let mut cache = self
            .cache
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire cache lock: {}", e)))?;

        cache.pop(hash);
        Ok(())
    }

    /// Clear cache
    pub fn clear(&self) -> Result<()> {
        debug!("Clearing block cache");

        let mut cache = self
            .cache
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire cache lock: {}", e)))?;

        cache.clear();
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let cache = self
            .cache
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire cache lock: {}", e)))?;

        let hits = *self
            .hits
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire hits lock: {}", e)))?;

        let misses = *self
            .misses
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire misses lock: {}", e)))?;

        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64) / (total as f64)
        } else {
            0.0
        };

        Ok(CacheStats {
            hits,
            misses,
            hit_rate,
            current_size: cache.len(),
            max_size: cache.cap().get(),
        })
    }
}

/// Transaction cache with LRU eviction
pub struct TransactionCache {
    cache: Arc<Mutex<LruCache<String, CachedTransaction>>>,
    hits: Arc<Mutex<u64>>,
    misses: Arc<Mutex<u64>>,
}

impl TransactionCache {
    /// Create a new transaction cache
    pub fn new(capacity: usize) -> Result<Self> {
        let cache_size = NonZeroUsize::new(capacity)
            .ok_or_else(|| Error::InvalidData("Cache capacity must be positive".to_string()))?;

        Ok(Self {
            cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        })
    }

    /// Get transaction from cache
    pub fn get(&self, txid: &str) -> Result<Option<CachedTransaction>> {
        debug!("Getting transaction from cache: {}", txid);

        let mut cache = self
            .cache
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire cache lock: {}", e)))?;

        if let Some(tx) = cache.get(txid) {
            let mut hits = self
                .hits
                .lock()
                .map_err(|e| Error::Internal(format!("Failed to acquire hits lock: {}", e)))?;
            *hits += 1;
            Ok(Some(tx.clone()))
        } else {
            let mut misses = self
                .misses
                .lock()
                .map_err(|e| Error::Internal(format!("Failed to acquire misses lock: {}", e)))?;
            *misses += 1;
            Ok(None)
        }
    }

    /// Put transaction in cache
    pub fn put(&self, txid: String, tx: CachedTransaction) -> Result<()> {
        debug!("Putting transaction in cache: {}", txid);

        let mut cache = self
            .cache
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire cache lock: {}", e)))?;

        cache.put(txid, tx);
        Ok(())
    }

    /// Remove transaction from cache
    pub fn remove(&self, txid: &str) -> Result<()> {
        debug!("Removing transaction from cache: {}", txid);

        let mut cache = self
            .cache
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire cache lock: {}", e)))?;

        cache.pop(txid);
        Ok(())
    }

    /// Clear cache
    pub fn clear(&self) -> Result<()> {
        debug!("Clearing transaction cache");

        let mut cache = self
            .cache
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire cache lock: {}", e)))?;

        cache.clear();
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let cache = self
            .cache
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire cache lock: {}", e)))?;

        let hits = *self
            .hits
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire hits lock: {}", e)))?;

        let misses = *self
            .misses
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire misses lock: {}", e)))?;

        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64) / (total as f64)
        } else {
            0.0
        };

        Ok(CacheStats {
            hits,
            misses,
            hit_rate,
            current_size: cache.len(),
            max_size: cache.cap().get(),
        })
    }
}

/// UTXO cache with LRU eviction
pub struct UTXOCache {
    cache: Arc<Mutex<LruCache<String, CachedUTXO>>>,
    hits: Arc<Mutex<u64>>,
    misses: Arc<Mutex<u64>>,
}

impl UTXOCache {
    /// Create a new UTXO cache
    pub fn new(capacity: usize) -> Result<Self> {
        let cache_size = NonZeroUsize::new(capacity)
            .ok_or_else(|| Error::InvalidData("Cache capacity must be positive".to_string()))?;

        Ok(Self {
            cache: Arc::new(Mutex::new(LruCache::new(cache_size))),
            hits: Arc::new(Mutex::new(0)),
            misses: Arc::new(Mutex::new(0)),
        })
    }

    /// Get UTXO from cache
    pub fn get(&self, key: &str) -> Result<Option<CachedUTXO>> {
        debug!("Getting UTXO from cache: {}", key);

        let mut cache = self
            .cache
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire cache lock: {}", e)))?;

        if let Some(utxo) = cache.get(key) {
            let mut hits = self
                .hits
                .lock()
                .map_err(|e| Error::Internal(format!("Failed to acquire hits lock: {}", e)))?;
            *hits += 1;
            Ok(Some(utxo.clone()))
        } else {
            let mut misses = self
                .misses
                .lock()
                .map_err(|e| Error::Internal(format!("Failed to acquire misses lock: {}", e)))?;
            *misses += 1;
            Ok(None)
        }
    }

    /// Put UTXO in cache
    pub fn put(&self, key: String, utxo: CachedUTXO) -> Result<()> {
        debug!("Putting UTXO in cache: {}", key);

        let mut cache = self
            .cache
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire cache lock: {}", e)))?;

        cache.put(key, utxo);
        Ok(())
    }

    /// Remove UTXO from cache
    pub fn remove(&self, key: &str) -> Result<()> {
        debug!("Removing UTXO from cache: {}", key);

        let mut cache = self
            .cache
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire cache lock: {}", e)))?;

        cache.pop(key);
        Ok(())
    }

    /// Clear cache
    pub fn clear(&self) -> Result<()> {
        debug!("Clearing UTXO cache");

        let mut cache = self
            .cache
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire cache lock: {}", e)))?;

        cache.clear();
        Ok(())
    }

    /// Get cache statistics
    pub fn stats(&self) -> Result<CacheStats> {
        let cache = self
            .cache
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire cache lock: {}", e)))?;

        let hits = *self
            .hits
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire hits lock: {}", e)))?;

        let misses = *self
            .misses
            .lock()
            .map_err(|e| Error::Internal(format!("Failed to acquire misses lock: {}", e)))?;

        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64) / (total as f64)
        } else {
            0.0
        };

        Ok(CacheStats {
            hits,
            misses,
            hit_rate,
            current_size: cache.len(),
            max_size: cache.cap().get(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_cache_creation() {
        let cache = BlockCache::new(100).unwrap();
        let stats = cache.stats().unwrap();
        assert_eq!(stats.current_size, 0);
        assert_eq!(stats.max_size, 100);
    }

    #[test]
    fn test_block_cache_put_get() {
        let cache = BlockCache::new(100).unwrap();
        let block = CachedBlock {
            hash: "a".repeat(128),
            height: 100,
            data: vec![1, 2, 3],
            cached_at: 1234567890,
        };

        cache.put(block.hash.clone(), block.clone()).unwrap();
        let retrieved = cache.get(&block.hash).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().height, 100);
    }

    #[test]
    fn test_block_cache_remove() {
        let cache = BlockCache::new(100).unwrap();
        let block = CachedBlock {
            hash: "a".repeat(128),
            height: 100,
            data: vec![1, 2, 3],
            cached_at: 1234567890,
        };

        cache.put(block.hash.clone(), block.clone()).unwrap();
        cache.remove(&block.hash).unwrap();
        let retrieved = cache.get(&block.hash).unwrap();
        assert!(retrieved.is_none());
    }

    #[test]
    fn test_transaction_cache_creation() {
        let cache = TransactionCache::new(100).unwrap();
        let stats = cache.stats().unwrap();
        assert_eq!(stats.current_size, 0);
        assert_eq!(stats.max_size, 100);
    }

    #[test]
    fn test_utxo_cache_creation() {
        let cache = UTXOCache::new(100).unwrap();
        let stats = cache.stats().unwrap();
        assert_eq!(stats.current_size, 0);
        assert_eq!(stats.max_size, 100);
    }

    #[test]
    fn test_cache_hit_rate() {
        let cache = BlockCache::new(100).unwrap();
        let block = CachedBlock {
            hash: "a".repeat(128),
            height: 100,
            data: vec![1, 2, 3],
            cached_at: 1234567890,
        };

        cache.put(block.hash.clone(), block.clone()).unwrap();
        cache.get(&block.hash).unwrap(); // Hit
        cache.get(&"b".repeat(128)).unwrap(); // Miss

        let stats = cache.stats().unwrap();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate, 0.5);
    }
}
