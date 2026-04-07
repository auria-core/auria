//! End-to-end integration tests for AURIA
//! Tests the full pipeline from input to output

use auria_execution::*;
use auria_router::*;
use auria_tensor::*;
use auria_core::*;

#[test]
fn test_full_inference_pipeline_standard() {
    let input = create_tensor_from_vec(&[1, 128], vec![0.5; 128]);
    
    let normalized = layer_norm(tensor_to_vec(&input).unwrap().as_slice(), 128, 1e-5);
    assert_eq!(normalized.len(), 128);
    
    let activated = relu(&normalized);
    assert!(activated.iter().all(|&x| x >= 0.0));
    
    let output = softmax(&activated);
    let sum: f32 = output.iter().sum();
    assert!((sum - 1.0).abs() < 0.01);
}

#[test]
fn test_full_inference_pipeline_pro() {
    let input = create_tensor_from_vec(&[1, 256], vec![0.3; 256]);
    
    let normalized = layer_norm(tensor_to_vec(&input).unwrap().as_slice(), 256, 1e-5);
    let activated = gelu(&normalized);
    
    let output = softmax(&activated);
    let max_prob = output.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    
    assert!(max_prob > 0.0);
    assert!(max_prob <= 1.0);
}

#[test]
fn test_routing_to_execution_pipeline() {
    let router = DeterministicRouter::new(1024);
    let decision = router.route(Tier::Standard, 42);
    
    assert!(!decision.expert_ids.is_empty());
    
    let embedding = TokenEmbedding::new(10000, 128);
    let token_emb = embedding.embed_token(42);
    assert_eq!(token_emb.len(), 128);
    
    let gating = GatingNetwork::new(8, 2);
    let gates = gating.compute_gates(&token_emb);
    
    assert!(!gates.is_empty());
}

#[test]
fn test_moe_integration() {
    let num_experts = 8;
    let embedding = TokenEmbedding::new(5000, 256);
    let gating = GatingNetwork::new(num_experts, 2);
    
    let input_token = embedding.embed_token(100);
    let gates = gating.compute_gates(&input_token);
    
    let mut expert_outputs: Vec<Vec<f32>> = Vec::new();
    for _ in 0..num_experts {
        let expert_input = tensor_to_vec(&create_tensor_from_vec(&[1, 256], input_token.clone())).unwrap();
        let activated = relu(&expert_input);
        expert_outputs.push(activated);
    }
    
    let mut combined = vec![0.0f32; 256];
    for (expert_idx, weight) in gates {
        if expert_idx < expert_outputs.len() {
            for (i, val) in expert_outputs[expert_idx].iter().enumerate() {
                combined[i] += val * weight;
            }
        }
    }
    
    assert_eq!(combined.len(), 256);
    assert!(combined.iter().all(|x| x.is_finite()));
}

#[test]
fn test_tensor_precision_pipeline() {
    let f32_data: Vec<f32> = (0..1024).map(|i| i as f32 / 1024.0).collect();
    
    let f16_data = convert_fp32_to_fp16(&f32_data);
    assert_eq!(f16_data.len(), f32_data.len() * 2);
    
    let recovered = convert_fp16_to_fp32(&f16_data).unwrap();
    
    let max_error: f32 = f32_data.iter()
        .zip(recovered.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);
    
    assert!(max_error < 0.1, "Max conversion error {} too high", max_error);
}

#[test]
fn test_attention_pipeline() {
    let config = AttentionConfig::default();
    
    let seq_len = 4;
    let head_dim = config.head_dim;
    let q = vec![0.1f32; seq_len * head_dim];
    let k = vec![0.1f32; seq_len * head_dim];
    let v = vec![0.2f32; seq_len * head_dim];
    
    let output = multihead_attention(&q, &k, &v, seq_len, head_dim, config.num_heads);
    
    assert_eq!(output.len(), seq_len * head_dim);
    assert!(output.iter().all(|x| x.is_finite()));
}

#[test]
fn test_kv_cache_pipeline() {
    let mut cache = KvCache::new(10, 64);
    
    for i in 0..5 {
        let keys = vec![i as f32; 64];
        let values = vec![i as f32 * 2.0; 64];
        cache.append(&keys, &values);
    }
    
    assert_eq!(cache.len(), 5);
    
    let (keys, values) = cache.as_tensors();
    assert!(!keys.data.is_empty());
    assert!(!values.data.is_empty());
}

#[test]
fn test_config_persistence() {
    let config = ExecutionConfig {
        max_batch_size: 16,
        enable_caching: true,
        cache_size: 20,
        timeout_seconds: 300,
        enable_moe: true,
        moe_top_k: 4,
    };
    
    let serialized = serde_json::to_string(&config).unwrap();
    let deserialized: ExecutionConfig = serde_json::from_str(&serialized).unwrap();
    
    assert_eq!(config.max_batch_size, deserialized.max_batch_size);
    assert_eq!(config.enable_moe, deserialized.enable_moe);
}

#[tokio::test]
async fn test_async_execution_pipeline() {
    struct TestBackend;
    
    #[async_trait]
    impl ExecutionBackend for TestBackend {
        async fn execute_step(
            &self,
            input: Tensor,
            _experts: Vec<Tensor>,
            _state: ExecutionState,
        ) -> AuriaResult<ExecutionOutput> {
            Ok(ExecutionOutput {
                tokens: vec!["test".to_string()],
                usage: UsageStats { tokens_generated: 10 },
            })
        }
        
        fn backend_name(&self) -> &str { "test" }
        fn supported_tiers(&self) -> &[Tier] { &[Tier::Standard] }
    }
    
    let backend = TestBackend;
    let engine = ExecutionEngine::new(backend);
    
    let input = create_tensor_from_vec(&[1, 128], vec![0.5; 128]);
    let routing = RoutingDecision {
        expert_ids: vec![ExpertId([1u8; 32])],
    };
    
    let result = engine.execute(input, routing, ExecutionState::default()).await;
    assert!(result.is_ok());
}
