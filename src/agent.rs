use crate::{
    config::AppConfig,
    models::{ChatCompletionRequest, ChatCompletionResponse, ChatMessage, Choice, Usage, new_id, Tier},
    node_client::{NodeClient, NodeGenerateRequest},
    policy::PolicyEngine,
    routing::{NodePool, NodeRouter, RoundRobinRouter},
};
use time::OffsetDateTime;

#[derive(Clone)]
pub struct AuriaAgent {
    cfg: AppConfig,
    policy: PolicyEngine,
    pool: NodePool,
    router: std::sync::Arc<dyn NodeRouter>,
}

impl AuriaAgent {
    pub async fn new(cfg: AppConfig) -> anyhow::Result<Self> {
        let mut nodes = Vec::new();
        for u in &cfg.node_urls {
            nodes.push(NodeClient::new(u)?);
        }
        if nodes.is_empty() {
            anyhow::bail!("no node urls configured");
        }

        Ok(Self {
            policy: PolicyEngine { default_tier: cfg.default_tier, max_cost_microusdc: cfg.max_cost_microusdc },
            pool: NodePool { nodes },
            router: std::sync::Arc::new(RoundRobinRouter::default()),
            cfg,
        })
    }

    pub async fn check_nodes(&self) -> anyhow::Result<()> {
        for n in &self.pool.nodes {
            // Best-effort health check; in prod, collect metrics.
            let _ = n.healthz().await;
        }
        Ok(())
    }

    pub fn config(&self) -> &AppConfig { &self.cfg }

    pub async fn chat_completions(&self, req: ChatCompletionRequest) -> anyhow::Result<ChatCompletionResponse> {
        let requested_tier = parse_model_tier(&req.model).or(Some(self.cfg.default_tier));
        let pd = self.policy.decide(requested_tier, req.max_tokens);

        if !pd.allowed {
            anyhow::bail!(pd.deny_reason.unwrap_or_else(|| "request denied".to_string()));
        }

        let prompt = messages_to_prompt(&req.messages);
        let idx = self.router.pick(pd.tier);
        let node = self.pool.get(idx);

        let node_resp = node.generate(NodeGenerateRequest {
            tier: pd.tier,
            prompt,
            max_tokens: pd.max_tokens,
        }).await?;

        let content = node_resp.tokens.join("");
        let created = OffsetDateTime::now_utc().unix_timestamp();

        Ok(ChatCompletionResponse {
            id: new_id(),
            created,
            model: req.model,
            choices: vec![Choice {
                index: 0,
                message: ChatMessage { role: "assistant".to_string(), content },
                finish_reason: "stop".to_string(),
            }],
            usage: Usage {
                prompt_tokens: 0,
                completion_tokens: node_resp.tokens_generated,
                total_tokens: node_resp.tokens_generated,
            },
        })
    }
}

fn messages_to_prompt(msgs: &[crate::models::ChatMessage]) -> String {
    // Production: apply prompt templates, system policies, tool calls, etc.
    let mut out = String::new();
    for m in msgs {
        out.push_str(&format!("{}: {}
", m.role, m.content));
    }
    out
}

fn parse_model_tier(model: &str) -> Option<Tier> {
    // Accept "AURIA:STANDARD" or "STANDARD"
    let m = model.trim();
    let parts: Vec<&str> = m.split(':').collect();
    if parts.len() == 2 {
        return Tier::parse(parts[1]);
    }
    Tier::parse(m)
}
