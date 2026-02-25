use crate::models::Tier;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyDecision {
    pub tier: Tier,
    pub max_tokens: u32,
    pub allowed: bool,
    pub deny_reason: Option<String>,
}

/// Minimal policy engine (production skeleton):
/// - tier parsing
/// - cost guard hooks
/// - request shaping
pub struct PolicyEngine {
    pub default_tier: Tier,
    pub max_cost_microusdc: u64,
}

impl PolicyEngine {
    pub fn decide(&self, requested_tier: Option<Tier>, max_tokens: Option<u32>) -> PolicyDecision {
        let tier = requested_tier.unwrap_or(self.default_tier);
        let max_tokens = max_tokens.unwrap_or(256).min(4096);

        // v1 skeleton: max cost is not computed (needs fee registry + receipts).
        if self.max_cost_microusdc > 0 {
            // Hook point for: estimate_cost(tier, max_tokens) > max_cost
        }

        PolicyDecision { tier, max_tokens, allowed: true, deny_reason: None }
    }
}
