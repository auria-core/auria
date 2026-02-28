// File: routing.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Node routing logic for distributing requests across
//     multiple Auria Nodes using round-robin or other strategies.
//
use crate::models::Tier;
use crate::node_client::NodeClient;

/// Routing strategy for selecting an Auria Node.
/// Production: use EWMA latency, capacity, tier support, stake, and reputation.
pub trait NodeRouter: Send + Sync {
    fn pick(&self, tier: Tier) -> usize;
}

/// Simple round-robin router.
#[derive(Default)]
pub struct RoundRobinRouter {
    next: std::sync::atomic::AtomicUsize,
}

impl NodeRouter for RoundRobinRouter {
    fn pick(&self, _tier: Tier) -> usize {
        self.next.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

#[derive(Clone)]
pub struct NodePool {
    pub nodes: Vec<NodeClient>,
}

impl NodePool {
    pub fn len(&self) -> usize {
        self.nodes.len()
    }
    pub fn get(&self, idx: usize) -> &NodeClient {
        &self.nodes[idx % self.nodes.len()]
    }
}
