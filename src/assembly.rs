// File: assembly.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Expert assembly subsystem for constructing executable expert tensors from shards.
//     Implements the Expert Assembler (AEA) as defined in the specification.
//
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use rand::Rng;
use sha2::{Sha256, Digest};
use crate::storage_interface::EnhancedModelStore;
use crate::license::LicenseManager;
use crate::execution::Device;
use crate::core::{Shard, ShardId, ExpertId, Tensor, TensorDType, DevicePointer};

#[derive(Debug, Clone)]
pub struct AssemblyRequest {
    pub expert_id: ExpertId,
    pub shard_ids: Vec<ShardId>,
    pub target_device: Device,
}

#[derive(Debug, Clone)]
pub struct AssemblyPlan {
    pub shard_order: Vec<ShardId>,
    pub total_elements: u64,
    pub tensor_shape: Vec<u32>,
    pub dtype: TensorDType,
}

#[derive(Debug, Clone)]
pub struct ExpertTensor {
    pub expert_id: ExpertId,
    pub device_pointer: DevicePointer,
    pub shape: Vec<u32>,
    pub dtype: TensorDType,
    pub watermark_applied: bool,
}

pub struct ExpertAssembler {
    model_store: Arc<Mutex<EnhancedModelStore>>,
    license_manager: Arc<Mutex<LicenseManager>>,
    expert_cache: Arc<Mutex<ExpertCache>>,
}

impl ExpertAssembler {
    pub fn new(
        model_store: Arc<Mutex<EnhancedModelStore>>,
        license_manager: Arc<Mutex<LicenseManager>>,
        expert_cache: Arc<Mutex<ExpertCache>>,
    ) -> Self {
        Self {
            model_store,
            license_manager,
            expert_cache,
        }
    }

    pub async fn assemble_expert(&self, request: AssemblyRequest) -> Result<ExpertTensor, AssemblyError> {
        // Check cache first
        if let Some(cached) = self.expert_cache.lock().await.get(&request.expert_id) {
            return Ok(cached.clone());
        }

        // Retrieve shards
        let mut shards = Vec::new();
        for shard_id in &request.shard_ids {
            match self.model_store.lock().await.load_shard(shard_id).await {
                Ok(shard) => shards.push(shard),
                Err(_) => return Err(AssemblyError::MissingShard(*shard_id)),
            }
        }

        // Validate licenses
        for shard in &shards {
            let license = {
                let guard = self.license_manager.lock().await;
                guard.licenses.get(&shard.id.0)
                    .ok_or(AssemblyError::InvalidLicense(shard.id.clone()))?
                    .clone()
            };

            // For validation, we need to call validate_license which takes &self on LicenseManager.
            // We can use a separate lock.
            if !self.license_manager.lock().await.validate_license(&license) {
                return Err(AssemblyError::InvalidLicense(shard.id.clone()));
            }
        }

        // Create assembly plan
        let plan = self.create_assembly_plan(&request, &shards).await?;

        // Construct tensor
        let tensor = self.construct_tensor(&plan, &shards)?;

        // Apply watermark
        let tensor_with_watermark = self.apply_watermark(&tensor, &request.expert_id).await?;

        // Upload to device
        let device_pointer = self.upload_to_device(&tensor_with_watermark, &request.target_device)?;

        // Create expert tensor
        let expert_tensor = ExpertTensor {
            expert_id: request.expert_id.clone(),
            device_pointer,
            shape: plan.tensor_shape.clone(),
            dtype: plan.dtype.clone(),
            watermark_applied: true,
        };

        // Store in cache
        self.expert_cache.lock().await.insert(expert_tensor.clone());

        Ok(expert_tensor)
    }

    async fn create_assembly_plan(&self, request: &AssemblyRequest, shards: &[Shard]) -> Result<AssemblyPlan, AssemblyError> {
        // Get expert definition (clone to avoid lifetime issues)
        let expert_definition = {
            let guard = self.model_store.lock().await;
            guard.model_store.get_expert_definition(&request.expert_id)
                .ok_or(AssemblyError::UnknownExpert(request.expert_id.clone()))?
                .clone()
        };

        // Verify shard count matches
        if expert_definition.shard_ids.len() != shards.len() {
            return Err(AssemblyError::ShardCountMismatch);
        }

        // Create deterministic shard order based on expert definition
        let shard_order: Vec<ShardId> = expert_definition.shard_ids.clone();

        // Calculate total elements and tensor shape
        let mut total_elements: u64 = 0;
        let tensor_shape = expert_definition.tensor_layout.shape.clone();
        let dtype = expert_definition.tensor_layout.dtype.clone();

        for shard in shards {
            total_elements += shard.tensor.data.len() as u64;
        }

        Ok(AssemblyPlan {
            shard_order,
            total_elements,
            tensor_shape,
            dtype,
        })
    }

    fn construct_tensor(&self, plan: &AssemblyPlan, shards: &[Shard]) -> Result<Tensor, AssemblyError> {
        // Allocate contiguous memory
        let total_size = plan.total_elements as usize;
        let mut data = Vec::with_capacity(total_size);

        // Concatenate shard tensors in order
        for shard_id in &plan.shard_order {
            if let Some(shard) = shards.iter().find(|s| s.id.0 == shard_id.0) {
                data.extend_from_slice(&shard.tensor.data);
            } else {
                return Err(AssemblyError::MissingShard(*shard_id));
            }
        }

        Ok(Tensor {
            data,
            dimensions: plan.tensor_shape.clone(),
        })
    }

    async fn apply_watermark(&self, tensor: &Tensor, expert_id: &ExpertId) -> Result<Tensor, AssemblyError> {
        // Generate deterministic watermark based on node identity and expert ID
        let mut hasher = Sha256::new();
        hasher.update(&self.license_manager.lock().await.node_identity);
        hasher.update(&expert_id.0);
        let hash = hasher.finalize();

        // Apply minimal perturbation to a few elements
        let mut perturbed_tensor = tensor.clone();
        let mut rng = rand::thread_rng();

        for i in 0..3 { // Apply to 3 random elements
            let index = rng.gen_range(0..tensor.data.len());
            let perturbation = (hash[i] as f32 - 128.0) / 1000.0; // Small perturbation
            perturbed_tensor.data[index] += perturbation;
        }

        Ok(perturbed_tensor)
    }

    fn upload_to_device(&self, tensor: &Tensor, target_device: &Device) -> Result<DevicePointer, AssemblyError> {
        // In production, this would upload to actual device memory
        // For now, we'll create a mock device pointer
        Ok(DevicePointer {
            address: tensor.data.as_ptr() as usize,
            device_type: target_device.device_type.clone(),
        })
    }
}

#[derive(Debug)]
pub enum AssemblyError {
    MissingShard(ShardId),
    InvalidLicense(ShardId),
    UnknownExpert(ExpertId),
    ShardCountMismatch,
    MemoryAllocationError,
    DeviceUploadError,
    WatermarkApplicationError,
}

impl std::fmt::Display for AssemblyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssemblyError::MissingShard(id) => write!(f, "Missing shard: {:?}", id),
            AssemblyError::InvalidLicense(id) => write!(f, "Invalid license for shard: {:?}", id),
            AssemblyError::UnknownExpert(id) => write!(f, "Unknown expert: {:?}", id),
            AssemblyError::ShardCountMismatch => write!(f, "Shard count mismatch"),
            AssemblyError::MemoryAllocationError => write!(f, "Memory allocation failed"),
            AssemblyError::DeviceUploadError => write!(f, "Device upload failed"),
            AssemblyError::WatermarkApplicationError => write!(f, "Watermark application failed"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExpertCache {
    cache: HashMap<ExpertId, ExpertTensor>,
    max_size: usize,
}

impl ExpertCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: HashMap::new(),
            max_size,
        }
    }

    pub fn get(&self, expert_id: &ExpertId) -> Option<&ExpertTensor> {
        self.cache.get(expert_id)
    }

    pub fn insert(&mut self, expert_tensor: ExpertTensor) {
        if self.cache.len() >= self.max_size {
            // Evict least recently used (simplified for now)
            let oldest_key = self.cache.keys().next().cloned();
            if let Some(key) = oldest_key {
                self.cache.remove(&key);
            }
        }
        self.cache.insert(expert_tensor.expert_id.clone(), expert_tensor);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Shard, ShardId, ExpertId, Tensor, TensorDType, ModelStore};

    #[tokio::test]
    async fn test_expert_assembly() {
        // Create test data
        let shard1 = Shard {
            id: ShardId([1u8; 32]),
            tensor: Tensor {
                data: vec![1.0, 2.0, 3.0],
                dimensions: vec![3],
            },
            metadata: ShardMetadata {
                shard_order: 1,
                dtype: TensorDType::FP32,
                dimensions: vec![3],
                creation_timestamp: 1000,
            },
        };

        let shard2 = Shard {
            id: ShardId([2u8; 32]),
            tensor: Tensor {
                data: vec![4.0, 5.0, 6.0],
                dimensions: vec![3],
            },
            metadata: ShardMetadata {
                shard_order: 2,
                dtype: TensorDType::FP32,
                dimensions: vec![3],
                creation_timestamp: 1000,
            },
        };

        // Create model store
        let mut model_store = ModelStore::new();
        model_store.store_shard(shard1.clone());
        model_store.store_shard(shard2.clone());

        let expert_id = ExpertId([3u8; 32]);
        let expert_definition = ExpertDefinition {
            id: expert_id.clone(),
            shard_ids: vec![shard1.id.clone(), shard2.id.clone()],
            tensor_layout: TensorLayout {
                shape: vec![2, 3],
                strides: vec![3, 1],
                dtype: TensorDType::FP32,
            },
        };
        model_store.add_expert_definition(expert_definition);

        // Create enhanced model store
        let enhanced_model_store = EnhancedModelStore {
            model_store,
            storage: Storage::new(StorageConfig::new(StorageBackend::Memory, 100)).await.unwrap(),
        };

        // Create license manager
        let mut license_manager = LicenseManager::new([0u8; 32]);
        license_manager.add_license(License {
            shard_id: shard1.id.0,
            node_pubkey: [0u8; 32],
            expiry_timestamp: 4102444800, // Year 2100
            signature: [0u8; 64],
        });

        license_manager.add_license(License {
            shard_id: shard2.id.0,
            node_pubkey: [0u8; 32],
            expiry_timestamp: 4102444800,
            signature: [0u8; 64],
        });

        // Create assembler
        let assembler = ExpertAssembler {
            model_store: Arc::new(Mutex::new(enhanced_model_store)),
            license_manager: Arc::new(Mutex::new(license_manager)),
            expert_cache: Arc::new(Mutex::new(ExpertCache::new(10))),
        };

        // Create assembly request
        let request = AssemblyRequest {
            expert_id: expert_id.clone(),
            shard_ids: vec![shard1.id.clone(), shard2.id.clone()],
            target_device: Device {
                device_type: DeviceType::CPU,
                memory_capacity: 1024,
            },
        };

        // Test assembly
        let result = assembler.assemble_expert(request).await.unwrap();

        assert_eq!(result.expert_id, expert_id);
        assert_eq!(result.shape, vec![2, 3]);
        assert!(result.watermark_applied);
    }

    #[tokio::test]
    async fn test_assembly_cache() {
        // Create test data and assembler (similar to above)
        // ...

        // Create assembler with cache
        let assembler = ExpertAssembler {
            model_store: Arc::new(Mutex::new(EnhancedModelStore::new(StorageConfig::new(StorageBackend::Memory, 100)).await.unwrap())),
            license_manager: Arc::new(Mutex::new(LicenseManager::new([0u8; 32]))),
            expert_cache: Arc::new(Mutex::new(ExpertCache::new(10))),
        };

        // First assembly
        let request = AssemblyRequest {
            expert_id: ExpertId([1u8; 32]),
            shard_ids: vec![ShardId([1u8; 32])],
            target_device: Device { device_type: DeviceType::CPU, memory_capacity: 1024 },
        };

        let result1 = assembler.assemble_expert(request.clone()).await.unwrap();

        // Second assembly - should hit cache
        let result2 = assembler.assemble_expert(request).await.unwrap();

        assert_eq!(result1.expert_id, result2.expert_id);
    }
}