#[test]
fn parses_tier() {
    assert!(auria::models::Tier::parse("STANDARD").is_some());
    assert!(auria::models::Tier::parse("nano").is_some());
}
