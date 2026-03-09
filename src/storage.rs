// File: storage.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Storage subsystem for managing shard and expert storage lifecycle.
//     Implements the Model Store (AMS) as defined in the specification.
//
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;
use crate::storage_interface::{Storage, StorageConfig, StorageBackend};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShardId([u8; 32]);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExpertId([u8; 32]);

#[derive(Debug, Clone)]
pub struct Shard {
    pub id: ShardId,
    pub tensor: Tensor,
    pub metadata: ShardMetadata,
}

#[derive(Debug, Clone)]
pub struct ShardMetadata {
    pub shard_order: u32,
    pub dtype: TensorDType,
    pub dimensions: Vec<u32>,
    pub creation_timestamp: u64,
}

#[derive(Debug, Clone)]
pub struct ExpertDefinition {
    pub id: ExpertId,
    pub shard_ids: Vec<ShardId>,
    pub tensor_layout: TensorLayout,
}

#[derive(Debug, Clone)]
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

    pub async fn load_shard(&self, shard_id: &ShardId) -> Option<&Shard> {
        if let Some(shard) = self.shards.get(shard_id) {
            return Some(shard);
        }

        // Try to load from storage
        if let Ok(shard) = self.storage.get_shard(shard_id.clone()).await {
            self.shards.insert(shard_id.clone(), shard.clone());
            return Some(&self.shards[shard_id]);
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

#[derive(Debug, Clone)]
pub struct Tensor {
    pub data: Vec<f32>,
    pub dimensions: Vec<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TensorDType {
    FP32,
    FP16,
    INT8,
    INT4,
}

#[derive(Debug, Clone)]
pub struct TensorLayout {
    pub shape: Vec<u32>,
    pub strides: Vec<u32>,
    pub dtype: TensorDType,
}

#[derive(Debug, Clone)]
pub struct DevicePointer {
    pub address: usize,
    pub device_type: DeviceType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeviceType {
    CPU,
    CUDA,
    ROCm,
    Metal,
}