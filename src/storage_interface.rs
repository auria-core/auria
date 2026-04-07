// File: storage_interface.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Unified storage interface for AURIA Runtime Core.
//     Provides a consistent API across different storage backends.
//
use std::collections::HashMap;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use lru::LruCache;
use tokio::sync::RwLock;
use auria_core::{AuriaError, AuriaResult};
use crate::core::{Shard, ShardId};

// Storage backend types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorageBackend {
    File,
    Memory,
    // Placeholders for future backends
    Redis,
    S3,
    SQLite,
    PostgreSQL,
}

// Backend-specific configuration structs
#[derive(Debug, Clone)]
pub struct FileStorageConfig {
    pub root_path: PathBuf,
    pub max_size_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct MemoryStorageConfig {
    pub max_items: usize,
}

// Global storage configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub backend: StorageBackend,
    pub file: Option<FileStorageConfig>,
    pub memory: Option<MemoryStorageConfig>,
    pub cache_size: usize,
    pub ttl: Option<Duration>,
}

impl StorageConfig {
    // Constructor used in tests and examples
    pub fn new(backend: StorageBackend, cache_size: usize) -> Self {
        match backend {
            StorageBackend::File => Self {
                backend,
                file: Some(FileStorageConfig {
                    root_path: PathBuf::from("/data/shards"),
                    max_size_bytes: 10 * 1024 * 1024 * 1024, // 10GB
                }),
                memory: None,
                cache_size,
                ttl: None,
            },
            StorageBackend::Memory => Self {
                backend,
                file: None,
                memory: Some(MemoryStorageConfig { max_items: 1000 }),
                cache_size,
                ttl: None,
            },
            StorageBackend::Redis => Self {
                backend,
                file: None,
                memory: None,
                cache_size,
                ttl: None,
            },
            StorageBackend::S3 => Self {
                backend,
                file: None,
                memory: None,
                cache_size,
                ttl: None,
            },
            StorageBackend::SQLite => Self {
                backend,
                file: None,
                memory: None,
                cache_size,
                ttl: None,
            },
            StorageBackend::PostgreSQL => Self {
                backend,
                file: None,
                memory: None,
                cache_size,
                ttl: None,
            },
        }
    }
}

// Storage statistics
#[derive(Debug, Clone)]
pub struct StorageStats {
    pub backend: StorageBackend,
    pub total_shards: u64,
    pub cache_size: usize,
    pub cache_capacity: usize,
    pub last_error: Option<String>,
}

// Storage backend trait
#[async_trait]
pub trait StorageBackendTrait: Send + Sync {
    async fn get_shard(&self, shard_id: ShardId) -> AuriaResult<Shard>;
    async fn put_shard(&self, shard: Shard) -> AuriaResult<()>;
    async fn delete_shard(&self, shard_id: ShardId) -> AuriaResult<()>;
    async fn exists(&self, shard_id: ShardId) -> bool;
    async fn list_shards(&self, limit: Option<usize>) -> AuriaResult<Vec<ShardId>>;
    async fn get_stats(&self) -> StorageStats;
    async fn clear(&self) -> AuriaResult<()>;
}

// Storage struct with caching layer
#[derive(Clone)]
pub struct Storage {
    backend: Arc<dyn StorageBackendTrait>,
    cache: Arc<RwLock<LruCache<ShardId, Shard>>>,
    config: StorageConfig,
}

impl Storage {
    pub async fn new(config: StorageConfig) -> AuriaResult<Self> {
        let backend = Self::create_backend(config.clone()).await?;
        let cache = Arc::new(RwLock::new(
            LruCache::new(
                NonZeroUsize::new(config.cache_size.max(1)).unwrap()
            )
        ));
        Ok(Self { backend, cache, config })
    }

    async fn create_backend(config: StorageConfig) -> AuriaResult<Arc<dyn StorageBackendTrait>> {
        match config.backend {
            StorageBackend::File => {
                let file_config = config.file.clone().unwrap_or_else(||
                    FileStorageConfig {
                        root_path: PathBuf::from("/data/shards"),
                        max_size_bytes: 10 * 1024 * 1024 * 1024,
                    }
                );
                Ok(Arc::new(FileStorage::new(file_config, config.cache_size)))
            }
            StorageBackend::Memory => {
                let memory_config = config.memory.clone().unwrap_or_else(||
                    MemoryStorageConfig { max_items: 1000 }
                );
                Ok(Arc::new(MemoryStorage::new(memory_config.max_items)))
            }
            _ => Err(AuriaError::ConfigError(format!("Storage backend {:?} not implemented", config.backend))),
        }
    }

    pub async fn get_shard(&self, shard_id: ShardId) -> AuriaResult<Shard> {
        // Check local cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(shard) = cache.get(&shard_id) {
                return Ok(shard.clone());
            }
        }
        // Fetch from backend
        let shard = self.backend.get_shard(shard_id).await?;
        // Update local cache
        {
            let mut cache = self.cache.write().await;
            if cache.len() >= self.config.cache_size {
                cache.pop_lru();
            }
            cache.put(shard_id, shard.clone());
        }
        Ok(shard)
    }

    pub async fn put_shard(&self, shard: Shard) -> AuriaResult<()> {
        // Store in backend
        self.backend.put_shard(shard.clone()).await?;
        // Update local cache
        {
            let mut cache = self.cache.write().await;
            if cache.len() >= self.config.cache_size {
                cache.pop_lru();
            }
            cache.put(shard.id, shard);
        }
        Ok(())
    }

    pub async fn delete_shard(&self, shard_id: ShardId) -> AuriaResult<()> {
        // Delete from backend
        self.backend.delete_shard(shard_id).await?;
        // Remove from local cache
        {
            let mut cache = self.cache.write().await;
            cache.pop(&shard_id);
        }
        Ok(())
    }

    pub async fn exists(&self, shard_id: ShardId) -> bool {
        // Check local cache first
        {
            let cache = self.cache.read().await;
            if cache.contains(&shard_id) {
                return true;
            }
        }
        // Check backend
        self.backend.exists(shard_id).await
    }

    pub async fn list_shards(&self, limit: Option<usize>) -> AuriaResult<Vec<ShardId>> {
        self.backend.list_shards(limit).await
    }

    pub async fn get_stats(&self) -> StorageStats {
        self.backend.get_stats().await
    }

    pub async fn clear(&self) -> AuriaResult<()> {
        // Clear local cache
        {
            let mut cache = self.cache.write().await;
            cache.clear();
        }
        // Clear backend
        self.backend.clear().await
    }
}

// File storage implementation
pub struct FileStorage {
    config: FileStorageConfig,
    cache: Arc<RwLock<LruCache<ShardId, Shard>>>,
    max_cache_size: usize,
}

impl FileStorage {
    pub fn new(config: FileStorageConfig, cache_size: usize) -> Self {
        let cache = Arc::new(RwLock::new(
            LruCache::new(NonZeroUsize::new(cache_size.max(1)).unwrap())
        ));
        Self {
            config,
            cache,
            max_cache_size: cache_size,
        }
    }
}

#[async_trait]
impl StorageBackendTrait for FileStorage {
    async fn get_shard(&self, shard_id: ShardId) -> AuriaResult<Shard> {
        // Check cache first
        {
            let mut cache = self.cache.write().await;
            if let Some(shard) = cache.get(&shard_id) {
                return Ok(shard.clone());
            }
        }
        let path = self.config.root_path.join(hex::encode(shard_id.0));
        if !path.exists() {
            return Err(AuriaError::ShardNotFound(shard_id.0));
        }
        let data = tokio::fs::read(&path).await
            .map_err(|e| AuriaError::StorageError(format!("Failed to read shard from file: {}", e)))?;
        let shard: Shard = serde_json::from_slice(&data)
            .map_err(|e| AuriaError::SerializationError(format!("Failed to deserialize shard: {}", e)))?;
        {
            let mut cache = self.cache.write().await;
            if cache.len() >= self.max_cache_size {
                cache.pop_lru();
            }
            cache.put(shard_id, shard.clone());
        }
        Ok(shard)
    }

    async fn put_shard(&self, shard: Shard) -> AuriaResult<()> {
        let path = self.config.root_path.join(hex::encode(shard.id.0));
        tokio::fs::create_dir_all(&self.config.root_path).await.ok();
        let data = serde_json::to_vec(&shard)
            .map_err(|e| AuriaError::SerializationError(format!("Failed to serialize shard: {}", e)))?;
        tokio::fs::write(&path, data).await
            .map_err(|e| AuriaError::StorageError(format!("Failed to write shard to file: {}", e)))?;
        {
            let mut cache = self.cache.write().await;
            if cache.len() >= self.max_cache_size {
                cache.pop_lru();
            }
            cache.put(shard.id, shard);
        }
        Ok(())
    }

    async fn delete_shard(&self, shard_id: ShardId) -> AuriaResult<()> {
        let path = self.config.root_path.join(hex::encode(shard_id.0));
        if path.exists() {
            tokio::fs::remove_file(path).await
                .map_err(|e| AuriaError::StorageError(format!("Failed to delete shard file: {}", e)))?;
        }
        {
            let mut cache = self.cache.write().await;
            cache.pop(&shard_id);
        }
        Ok(())
    }

    async fn exists(&self, shard_id: ShardId) -> bool {
        {
            let cache = self.cache.read().await;
            if cache.contains(&shard_id) {
                return true;
            }
        }
        let path = self.config.root_path.join(hex::encode(shard_id.0));
        path.exists()
    }

    async fn list_shards(&self, limit: Option<usize>) -> AuriaResult<Vec<ShardId>> {
        let mut shard_ids = Vec::new();
        let mut dir = tokio::fs::read_dir(&self.config.root_path).await
            .map_err(|e| AuriaError::StorageError(format!("Failed to read directory: {}", e)))?;

        while let Some(entry) = dir.next_entry().await
            .map_err(|e| AuriaError::StorageError(format!("Failed to read dir entry: {}", e)))?
        {
            let file_name = entry.file_name();
            if let Some(name) = file_name.to_str() {
                if name.ends_with(".json") {
                    let id_str = name.strip_suffix(".json").unwrap_or(name);
                    if let Ok(id_bytes) = hex::decode(id_str) {
                        if id_bytes.len() == 32 {
                            if let Ok(id_array) = id_bytes.try_into() {
                                shard_ids.push(ShardId(id_array));
                            }
                        }
                    }
                }
            }
        }

        if let Some(limit) = limit {
            shard_ids.truncate(limit);
        }
        Ok(shard_ids)
    }

    async fn get_stats(&self) -> StorageStats {
        let cache = self.cache.read().await;
        StorageStats {
            backend: StorageBackend::File,
            total_shards: 0, // File storage doesn't track total count accurately
            cache_size: cache.len(),
            cache_capacity: self.max_cache_size,
            last_error: None,
        }
    }

    async fn clear(&self) -> AuriaResult<()> {
        {
            let mut cache = self.cache.write().await;
            cache.clear();
        }
        if self.config.root_path.exists() {
            tokio::fs::remove_dir_all(&self.config.root_path).await
                .map_err(|e| AuriaError::StorageError(format!("Failed to clear file storage: {}", e)))?;
        }
        Ok(())
    }
}

// Memory storage implementation
pub struct MemoryStorage {
    store: Arc<RwLock<HashMap<ShardId, Shard>>>,
    max_items: usize,
}

impl MemoryStorage {
    pub fn new(max_items: usize) -> Self {
        Self {
            store: Arc::new(RwLock::new(HashMap::new())),
            max_items,
        }
    }
}

#[async_trait]
impl StorageBackendTrait for MemoryStorage {
    async fn get_shard(&self, shard_id: ShardId) -> AuriaResult<Shard> {
        let store = self.store.read().await;
        if let Some(shard) = store.get(&shard_id) {
            return Ok(shard.clone());
        }
        Err(AuriaError::ShardNotFound(shard_id.0))
    }

    async fn put_shard(&self, shard: Shard) -> AuriaResult<()> {
        let mut store = self.store.write().await;
        if store.len() >= self.max_items {
            // Simple eviction: remove first entry (could use LRU)
            if let Some(key) = store.keys().next().cloned() {
                store.remove(&key);
            }
        }
        store.insert(shard.id, shard);
        Ok(())
    }

    async fn delete_shard(&self, shard_id: ShardId) -> AuriaResult<()> {
        let mut store = self.store.write().await;
        store.remove(&shard_id);
        Ok(())
    }

    async fn exists(&self, shard_id: ShardId) -> bool {
        self.store.read().await.contains_key(&shard_id)
    }

    async fn list_shards(&self, limit: Option<usize>) -> AuriaResult<Vec<ShardId>> {
        let store = self.store.read().await;
        let mut shard_ids: Vec<ShardId> = store.keys().cloned().collect();
        if let Some(limit) = limit {
            shard_ids.truncate(limit);
        }
        Ok(shard_ids)
    }

    async fn get_stats(&self) -> StorageStats {
        let store = self.store.read().await;
        StorageStats {
            backend: StorageBackend::Memory,
            total_shards: store.len() as u64,
            cache_size: store.len(),
            cache_capacity: self.max_items,
            last_error: None,
        }
    }

    async fn clear(&self) -> AuriaResult<()> {
        let mut store = self.store.write().await;
        store.clear();
        Ok(())
    }
}

// Enhanced Model Store combines in-memory model store with persistent storage
pub struct EnhancedModelStore {
    pub model_store: crate::core::ModelStore,
    storage: Storage,
}

impl EnhancedModelStore {
    pub async fn new(config: StorageConfig) -> AuriaResult<Self> {
        let storage = Storage::new(config).await?;
        let model_store = crate::core::ModelStore::new();
        Ok(Self { model_store, storage })
    }

    pub async fn load_shard(&mut self, shard_id: &ShardId) -> AuriaResult<Shard> {
        if let Some(shard) = self.model_store.load_shard(shard_id) {
            return Ok(shard);
        }
        let shard = self.storage.get_shard(*shard_id).await?;
        // Cache in memory for future reads.
        self.model_store.store_shard(shard.clone());
        Ok(shard)
    }

    pub async fn store_shard(&mut self, shard: Shard) -> AuriaResult<()> {
        self.model_store.store_shard(shard.clone());
        self.storage.put_shard(shard).await
    }

    pub async fn shard_exists(&self, shard_id: &ShardId) -> bool {
        self.model_store.shard_exists(shard_id) || self.storage.exists(*shard_id).await
    }

    pub async fn list_shards(&self, limit: Option<usize>) -> AuriaResult<Vec<ShardId>> {
        self.storage.list_shards(limit).await
    }

    pub async fn get_storage_stats(&self) -> StorageStats {
        self.storage.get_stats().await
    }

    pub async fn clear(&mut self) -> AuriaResult<()> {
        self.model_store.clear();
        self.storage.clear().await
    }
}
