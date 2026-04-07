// File: config.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Application configuration handling using Figment for
//     multi-source configuration (TOML, JSON, environment variables).
//
use figment::{
    providers::{Env, Format, Json, Serialized, Toml},
    value::Uncased,
    Figment,
};
use serde::{Deserialize, Serialize};

use crate::models::Tier;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    /// Bind address for the agent HTTP API.
    pub bind: String,

    /// Comma-separated list of Auria Node base URLs.
    pub node_urls: Vec<String>,

    /// Default tier if request doesn't specify.
    pub default_tier: Tier,

    /// Optional max cost guard in micro-USDC.
    /// 0 means "unlimited" in this skeleton.
    pub max_cost_microusdc: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            bind: "127.0.0.1:8787".to_string(),
            node_urls: vec!["http://127.0.0.1:8080".to_string()],
            default_tier: Tier::Standard,
            max_cost_microusdc: 0,
        }
    }
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        // Supports:
        // - auria.toml / auria.json (optional)
        // - env vars:
        //   AURIA_BIND
        //   AURIA_NODE_URLS (comma-separated)
        //   AURIA_DEFAULT_TIER
        //   AURIA_MAX_COST_MICROUSDC
        let fig = Figment::from(Serialized::from(AppConfig::default(), "auria"))
            .merge(Toml::file("auria.toml").nested())
            .merge(Json::file("auria.json").nested())
            .merge(
                Env::prefixed("AURIA_")
                    .map(|k| Uncased::new(k.as_str().to_ascii_lowercase()))
                    .split(","),
            );

        let mut cfg: AppConfig = fig.extract()?;

        // Env split(",") turns NODE_URLS into Vec<String> only if named node_urls.
        // Support the common "AURIA_NODE_URLS" explicitly if provided.
        if let Ok(v) = std::env::var("AURIA_NODE_URLS") {
            cfg.node_urls = v
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        if let Ok(v) = std::env::var("AURIA_BIND") {
            cfg.bind = v;
        }
        if let Ok(v) = std::env::var("AURIA_DEFAULT_TIER") {
            cfg.default_tier = Tier::parse(&v).unwrap_or(cfg.default_tier);
        }
        if let Ok(v) = std::env::var("AURIA_MAX_COST_MICROUSDC") {
            if let Ok(n) = v.parse::<u64>() {
                cfg.max_cost_microusdc = n;
            }
        }

        Ok(cfg)
    }
}
