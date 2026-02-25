use crate::models::Tier;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Clone, Debug)]
pub struct NodeClient {
    base: Url,
    http: reqwest::Client,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeGenerateRequest {
    pub tier: Tier,
    pub prompt: String,
    pub max_tokens: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NodeGenerateResponse {
    pub tokens: Vec<String>,
    pub tokens_generated: u32,
}

impl NodeClient {
    pub fn new(base: &str) -> anyhow::Result<Self> {
        Ok(Self {
            base: Url::parse(base)?,
            http: reqwest::Client::new(),
        })
    }

    pub async fn healthz(&self) -> anyhow::Result<()> {
        let u = self.base.join("healthz")?;
        let r = self.http.get(u).send().await?;
        if !r.status().is_success() {
            anyhow::bail!("node healthz failed: {}", r.status());
        }
        Ok(())
    }

    pub async fn generate(&self, req: NodeGenerateRequest) -> anyhow::Result<NodeGenerateResponse> {
        // Production integration:
        // - Use the Auria Node API (AURIA Runtime Core) endpoint here.
        // - In the ARC skeleton, we didn't include an HTTP server; this agent is ready for that integration.
        // For now, provide a local placeholder behavior to keep this crate self-contained.
        Ok(NodeGenerateResponse {
            tokens: vec![
                format!("[{:?}] ", req.tier),
                req.prompt,
                " (stubbed)".to_string(),
            ],
            tokens_generated: 3,
        })
    }
}
