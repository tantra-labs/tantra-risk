//! Exhaustive tests for the MCP tool-risk classification hook.

use vyapaar_core::{Decision, Exchange, InstrumentId, OrderSide, Quantity};
use vyapaar_risk::{ToolRiskHint, risk_classification};

fn any_instrument() -> InstrumentId {
    InstrumentId::new(Exchange::Nse, "RELIANCE")
}

#[test]
fn hold_is_read_only() {
    assert_eq!(
        risk_classification(&Decision::Hold { reason: None }),
        ToolRiskHint::ReadOnly
    );
    // Every Hold shape maps identically, reason or not.
    assert_eq!(
        risk_classification(&Decision::Hold {
            reason: Some("waiting for a signal".into()),
        }),
        ToolRiskHint::ReadOnly
    );
}

#[test]
fn place_order_is_destructive() {
    let decision = Decision::PlaceOrder {
        symbol: "RELIANCE".into(),
        side: OrderSide::Buy,
        qty: Quantity::from_u64(1),
        price_limit: None,
        algo_id: "test-algo".into(),
        rationale: None,
    };
    assert_eq!(risk_classification(&decision), ToolRiskHint::Destructive);
}

#[test]
fn modify_order_is_destructive() {
    let decision = Decision::ModifyOrder {
        instrument: any_instrument(),
        new_qty: Some(Quantity::from_u64(2)),
        new_price: None,
        algo_id: "test-algo".into(),
        rationale: None,
    };
    assert_eq!(risk_classification(&decision), ToolRiskHint::Destructive);
}

#[test]
fn cancel_order_is_destructive() {
    let decision = Decision::CancelOrder {
        instrument: any_instrument(),
        algo_id: Some("test-algo".into()),
        rationale: None,
    };
    assert_eq!(risk_classification(&decision), ToolRiskHint::Destructive);
}

#[test]
fn cancel_all_is_destructive() {
    let decision = Decision::CancelAll {
        algo_id: Some("test-algo".into()),
        reason: Some("emergency".into()),
    };
    assert_eq!(risk_classification(&decision), ToolRiskHint::Destructive);
}

#[test]
fn mcp_keys_match_annotation_vocabulary() {
    assert_eq!(ToolRiskHint::ReadOnly.mcp_key(), "readOnlyHint");
    assert_eq!(ToolRiskHint::Destructive.mcp_key(), "destructiveHint");
    assert_eq!(ToolRiskHint::Idempotent.mcp_key(), "idempotentHint");
}

#[test]
fn hint_is_serde_roundtrip_stable() {
    for hint in [
        ToolRiskHint::ReadOnly,
        ToolRiskHint::Destructive,
        ToolRiskHint::Idempotent,
    ] {
        let json = serde_json::to_string(&hint).unwrap();
        let back: ToolRiskHint = serde_json::from_str(&json).unwrap();
        assert_eq!(hint, back);
        assert_eq!(back.mcp_key(), hint.mcp_key());
    }
}
