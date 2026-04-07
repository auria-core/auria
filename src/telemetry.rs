// File: telemetry.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Tracing and logging initialization using tracing_subscriber.
//     Configures environment-based log filtering.
//
use tracing_subscriber::EnvFilter;

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::from_default_env()
                .add_directive("info".parse().unwrap())
                .add_directive("tower_http=info".parse().unwrap()),
        )
        .init();
}
