//! SEBI Algo ID Registry unit tests.
//!
//! Extracted from `src/algo_registry.rs` during Tier 1A migration per the
//! Parallel Test Directory Rule: zero `#[cfg(test)]` modules inside `src/`.

use vyapaar_risk::AlgoIdRegistry;

#[test]
fn test_register_and_check() {
    let mut registry = AlgoIdRegistry::new();
    assert!(registry.is_empty());

    registry.register("algo-1");
    assert_eq!(registry.len(), 1);
    assert!(registry.is_registered_and_active("algo-1").is_ok());
    assert!(registry.is_registered_and_active("algo-unknown").is_err());
}

#[test]
fn test_suspend_and_reactivate() {
    let mut registry = AlgoIdRegistry::new();
    registry.register("algo-1");

    registry.suspend("algo-1", "SEBI regulatory query");
    assert!(registry.is_registered_and_active("algo-1").is_err());

    registry.reactivate("algo-1");
    assert!(registry.is_registered_and_active("algo-1").is_ok());
}

#[test]
fn test_unregister() {
    let mut registry = AlgoIdRegistry::new();
    registry.register("algo-1");
    registry.unregister("algo-1");
    assert!(registry.is_empty());
    assert!(registry.is_registered_and_active("algo-1").is_err());
}
