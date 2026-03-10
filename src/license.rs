// File: license.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     License management subsystem for validating shard access authorization.
//     Implements the License Manager (ALM) as defined in the specification.
//
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use rand::Rng;
use auria_core::shard::{PublicKey, Signature};
use auria_security::verify_signature;

#[derive(Debug, Clone)]
pub struct License {
    pub shard_id: [u8; 32],
    pub node_pubkey: [u8; 32],
    pub expiry_timestamp: u64,
    pub signature: [u8; 64],
}

#[derive(Debug, Clone)]
pub struct LicenseManager {
    pub licenses: HashMap<[u8; 32], License>,
    pub node_identity: [u8; 32],
}

impl LicenseManager {
    pub fn new(node_identity: [u8; 32]) -> Self {
        Self {
            licenses: HashMap::new(),
            node_identity,
        }
    }

    pub fn validate_license(&self, license: &License) -> bool {
        // Verify signature
        if !self.verify_signature(license) {
            return false;
        }

        // Check expiry
        if license.expiry_timestamp < Self::current_timestamp() {
            return false;
        }

        // Verify node identity
        if license.node_pubkey != self.node_identity {
            return false;
        }

        true
    }

    pub fn license_valid_for_shard(&self, shard_id: &[u8; 32]) -> bool {
        self.licenses.contains_key(shard_id)
    }

    pub fn add_license(&mut self, license: License) {
        self.licenses.insert(license.shard_id.clone(), license);
    }

    fn verify_signature(&self, license: &License) -> bool {
        // Get the node's public key from the license itself
        let pubkey = PublicKey(license.node_pubkey);
        let sig = Signature(license.signature);

        let mut data = Vec::new();
        data.extend_from_slice(&license.shard_id);
        data.extend_from_slice(&license.node_pubkey);
        data.extend_from_slice(&license.expiry_timestamp.to_le_bytes());

        verify_signature(&pubkey, &data, &sig).unwrap_or(false)
    }

    fn current_timestamp() -> u64 {
        // Placeholder for current timestamp
        // In production, this would use a secure time source
        1726473600 + (rand::thread_rng().gen::<u32>() % 86400) as u64
    }
}