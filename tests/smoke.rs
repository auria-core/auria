// File: smoke.rs - This file is part of AURIA
// Copyright (c) 2026 AURIA Developers and Contributors
// Description:
//     Smoke tests for basic functionality including tier parsing
//     and configuration validation.
//
#[test]
fn parses_tier() {
    assert!(auria::models::Tier::parse("STANDARD").is_some());
    assert!(auria::models::Tier::parse("nano").is_some());
}
