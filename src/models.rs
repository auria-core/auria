// File: models.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Data models for OpenAI-compatible API types including
//     chat completions, messages, tiers, and usage tracking.
//     Extended with assembly and execution models.
//
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Tier {
    Nano,
    Standard,
    Pro,
    Max,
}

impl Tier {
    pub fn parse(s: &str) -> Option<Self> {
        match s.trim().to_ascii_uppercase().as_str() {
            "NANO" => Some(Tier::Nano),
            "STANDARD" => Some(Tier::Standard),
            "PRO" => Some(Tier::Pro),
            "MAX" => Some(Tier::Max),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// Example: "AURIA:STANDARD"
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    pub id: String,
    pub created: i64,
    pub model: String,
    pub choices: Vec<Choice>,
    pub usage: Usage,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Choice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

pub fn new_id() -> String {
    format!("auria_{}", Uuid::new_v4())
}

// Assembly Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyRequest {
    pub expert_id: String,
    pub shard_ids: Vec<String>,
    pub target_device: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssemblyPlan {
    pub shard_order: Vec<String>,
    pub total_elements: u64,
    pub tensor_shape: Vec<u32>,
    pub dtype: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertTensor {
    pub expert_id: String,
    pub device_pointer: String,
    pub shape: Vec<u32>,
    pub dtype: String,
    pub watermark_applied: bool,
}

// Execution Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRequest {
    pub expert_tensors: Vec<ExpertTensor>,
    pub input_tensor: Tensor,
    pub execution_context: ExecutionContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub output_tensor: Tensor,
    pub execution_time_us: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub device: Device,
    pub stream: ExecutionStream,
    pub backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionStream {
    pub stream_id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub device_type: String,
    pub memory_capacity: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tensor {
    pub data: Vec<f32>,
    pub dimensions: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevicePointer {
    pub address: usize,
    pub device_type: String,
}

// Storage Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shard {
    pub id: String,
    pub tensor: Tensor,
    pub metadata: ShardMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShardMetadata {
    pub shard_order: u32,
    pub dtype: String,
    pub dimensions: Vec<u32>,
    pub creation_timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpertDefinition {
    pub id: String,
    pub shard_ids: Vec<String>,
    pub tensor_layout: TensorLayout,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TensorLayout {
    pub shape: Vec<u32>,
    pub strides: Vec<u32>,
    pub dtype: String,
}

// License Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub shard_id: String,
    pub node_pubkey: String,
    pub expiry_timestamp: u64,
    pub signature: String,
}

// Policy Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub tier: String,
    pub max_tokens: u32,
    pub allowed: bool,
    pub deny_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEngine {
    pub default_tier: String,
    pub max_cost_microusdc: u64,
}

// Node Models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeClient {
    pub base: String,
    pub http: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGenerateRequest {
    pub tier: String,
    pub prompt: String,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGenerateResponse {
    pub tokens: Vec<String>,
    pub tokens_generated: u32,
}