// File: core.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Core data types for AURIA, independent of storage and execution backends.
//     Includes shard definitions, tensors, expert definitions, and an in-memory model store.
//
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use uuid::Uuid;

// Shard identifier (binary 32-byte)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShardId(pub [u8; 32]);

impl ShardId {
    pub fn new() -> Self {
        let uuid = Uuid::new_v4();
        let mut bytes = [0u8; 32];
        bytes[..16].copy_from_slice(&uuid.as_bytes()[..16]);
        Self(bytes)
    }
}

impl fmt::Display for ShardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

// Expert identifier (binary 32-byte)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExpertId(pub [u8; 32]);

impl ExpertId {
    pub fn new() -> Self {
        let uuid = Uuid::new_v4();
        let mut bytes = [0u8; 32];
        bytes[..16].copy_from_slice(&uuid.as_bytes()[..16]);
        Self(bytes)
    }
}

impl fmt::Display for ExpertId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

// Tensor data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tensor {
    pub data: Vec<f32>,
    pub dimensions: Vec<u32>,
}

// Tensor data type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TensorDType {
    FP32,
    FP16,
    INT8,
    INT4,
}

// Tensor layout (shape, strides, dtype)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorLayout {
    pub shape: Vec<u32>,
    pub strides: Vec<u32>,
    pub dtype: TensorDType,
}

// Shard metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardMetadata {
    pub shard_order: u32,
    pub dtype: TensorDType,
    pub dimensions: Vec<u32>,
    pub creation_timestamp: u64,
}

// Shard (unit of intelligence)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shard {
    pub id: ShardId,
    pub tensor: Tensor,
    pub metadata: ShardMetadata,
}

// Expert definition (assembly of shards)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertDefinition {
    pub id: ExpertId,
    pub shard_ids: Vec<ShardId>,
    pub tensor_layout: TensorLayout,
}

// Device pointer for execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePointer {
    pub address: usize,
    pub device_type: DeviceType,
}

// Device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceType {
    CPU,
    CUDA,
    ROCm,
    Metal,
}

// In-memory model store (no persistence)
#[derive(Debug, Clone)]
pub struct ModelStore {
    shards: HashMap<ShardId, Shard>,
    expert_definitions: HashMap<ExpertId, ExpertDefinition>,
}

impl ModelStore {
    pub fn new() -> Self {
        Self {
            shards: HashMap::new(),
            expert_definitions: HashMap::new(),
        }
    }

    pub fn store_shard(&mut self, shard: Shard) {
        self.shards.insert(shard.id, shard);
    }

    pub fn load_shard(&self, shard_id: &ShardId) -> Option<Shard> {
        self.shards.get(shard_id).cloned()
    }

    pub fn shard_exists(&self, shard_id: &ShardId) -> bool {
        self.shards.contains_key(shard_id)
    }

    pub fn get_expert_definition(&self, expert_id: &ExpertId) -> Option<&ExpertDefinition> {
        self.expert_definitions.get(expert_id)
    }

    pub fn add_expert_definition(&mut self, definition: ExpertDefinition) {
        self.expert_definitions.insert(definition.id, definition);
    }

    pub fn list_shards(&self) -> Vec<ShardId> {
        self.shards.keys().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.shards.clear();
        self.expert_definitions.clear();
    }
}
