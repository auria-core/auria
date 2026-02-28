// File: lib.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Core library entry point for the AURIA agent framework.
//     Exports all public modules and re-exports main types.
//
pub mod config;
pub mod models;
pub mod policy;
pub mod node_client;
pub mod routing;
pub mod agent;
pub mod api;
pub mod telemetry;

pub use agent::AuriaAgent;
