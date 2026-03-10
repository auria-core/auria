// File: storage.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Storage subsystem for managing shard and expert storage lifecycle.
//     Implements the persistent Model Store (AMS) as defined in the specification.
//     The core data types are defined in the `core` module.
//
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use auria_core::AuriaResult;
use crate::core::{Shard, ShardId, ExpertId, ExpertDefinition, TensorDType};
use crate::storage_interface::{Storage, StorageConfig, StorageBackend, StorageStats};

#[derive(Clone)]
pub struct ModelStore {
    shards: HashMap<ShardId, Shard>,
    expert_definitions: HashMap<ExpertId, ExpertDefinition>,
    storage: Storage,
}

impl ModelStore {
    pub async fn new(storage_config: StorageConfig) -> AuriaResult<Self> {
        let storage = Storage::new(storage_config).await?;
        Ok(Self {
            shards: HashMap::new(),
            expert_definitions: HashMap::new(),
            storage,
        })
    }

    pub async fn load_shard(&mut self, shard_id: &ShardId) -> Option<Shard> {
        // Check in-memory cache first
        if let Some(shard) = self.shards.get(shard_id) {
            return Some(shard.clone());
        }

        // Load from persistent storage
        if let Ok(shard) = self.storage.get_shard(shard_id.clone()).await {
            self.shards.insert(shard_id.clone(), shard.clone());
            return Some(shard);
        }

        None
    }

    pub async fn store_shard(&mut self, shard: Shard) -> AuriaResult<()> {
        self.shards.insert(shard.id.clone(), shard.clone());
        self.storage.put_shard(shard).await
    }

    pub async fn shard_exists(&self, shard_id: &ShardId) -> bool {
        self.shards.contains_key(shard_id) || self.storage.exists(shard_id.clone()).await
    }

    pub async fn get_expert_definition(&self, expert_id: &ExpertId) -> Option<&ExpertDefinition> {
        self.expert_definitions.get(expert_id)
    }

    pub async fn add_expert_definition(&mut self, definition: ExpertDefinition) {
        self.expert_definitions.insert(definition.id.clone(), definition);
    }

    pub async fn list_shards(&self, limit: Option<usize>) -> AuriaResult<Vec<ShardId>> {
        self.storage.list_shards(limit).await
    }

    pub async fn get_storage_stats(&self) -> StorageStats {
        self.storage.get_stats().await
    }

    pub async fn clear(&mut self) -> AuriaResult<()> {
        self.shards.clear();
        self.expert_definitions.clear();
        self.storage.clear().await
    }
}