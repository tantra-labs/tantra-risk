//! Integration tests for the sizing-gate weight / ADV20 caps and the
//! multi-currency refusal formatting fix (gap 0005).

use rust_decimal::Decimal;
use tantra_core::{
    Currency, Exchange, InstrumentId, Money, PositionSide, PositionSizingRequest, Price, Quantity,
    RiskBudget, SizingMethod,
};
use tantra_risk::sizing_gate::SizingRiskRefusal;
use tantra_risk::{RiskEngine, RiskError, validate_sizing_request};

const EQUITY: i64 = 1_000_000; // 10 Lakh account equity

/// Build a fixed-fractional sizing request (1% risk) over RELIANCE with the
/// given entry price, optional stop, and optional ADV20.
fn make_request(
    entry: Decimal,
    stop: Option<Decimal>,
    adv20: Option<Quantity>,
) -> (PositionSizingRequest, RiskBudget) {
    let instrument = InstrumentId::new(Exchange::Nse, "RELIANCE");
    let equity = Money::inr(Decimal::from(EQUITY));
    let budget = RiskBudget::new(
        equity,
        Decimal::from_str_radix("0.01", 10).unwrap(),
        Quantity::from_u64(1),
    );
    let mut request = PositionSizingRequest::new(
        instrument,
        PositionSide::Long,
        Price::new(entry),
        equity,
        budget.clone(),
        SizingMethod::FixedFractional {
            risk_pct: Decimal::from_str_radix("0.01", 10).unwrap(),
        },
    );
    if let Some(sl) = stop {
        request = request.with_stop_loss(Price::new(sl));
    }
    if let Some(a20) = adv20 {
        request = request.with_adv20(a20);
    }
    (request, budget)
}

#[test]
fn max_weight_exceeded_fires_when_notional_weight_breaches_cap() {
    // Tight stop (1 point) → raw quantity 10,000 → notional 25,000,000 →
    // weight 25. The cap lives on the request only, so the core sizing step
    // does not clamp it; the gate's independent re-check must refuse.
    let (mut request, budget) = make_request(Decimal::from(2500), Some(Decimal::from(2499)), None);
    request.risk_budget.max_weight_pct = Some(Decimal::from_str_radix("0.02", 10).unwrap());

    let res = validate_sizing_request(&request, &budget);
    match res {
        Err(RiskError::SizingRefusal(SizingRiskRefusal::MaxWeightExceeded {
            weight_pct,
            max_weight_pct,
        })) => {
            assert_eq!(max_weight_pct, Decimal::from_str_radix("0.02", 10).unwrap());
            assert!(weight_pct > max_weight_pct);
            assert_eq!(weight_pct, Decimal::from(25));
        }
        other => panic!("expected MaxWeightExceeded, got {other:?}"),
    }

    // The engine method path agrees (it delegates to the free function).
    let engine = RiskEngine::default();
    let res_engine = engine.validate_sizing_request(&request, &budget);
    assert!(matches!(
        res_engine,
        Err(RiskError::SizingRefusal(
            SizingRiskRefusal::MaxWeightExceeded { .. }
        ))
    ));
}

#[test]
fn max_weight_within_limits_passes() {
    // With the cap on the budget, the core clamps qty to
    // equity × 0.02 / 2500 = 8 → weight exactly 0.02 ≤ 0.02.
    let (mut request, mut budget) =
        make_request(Decimal::from(2500), Some(Decimal::from(2499)), None);
    budget.max_weight_pct = Some(Decimal::from_str_radix("0.02", 10).unwrap());
    request.risk_budget = budget.clone();

    let res = validate_sizing_request(&request, &budget);
    assert!(
        res.is_ok(),
        "expected within-weight-cap sizing to pass, got {res:?}"
    );
}

#[test]
fn max_adv_participation_fires_when_qty_breaches_adv_share() {
    // Raw quantity 10,000 over ADV20 100,000 = 10% participation vs a 1% cap.
    // The cap lives on the request only so the core does not clamp.
    let (mut request, budget) = make_request(
        Decimal::from(2500),
        Some(Decimal::from(2499)),
        Some(Quantity::from_u64(100_000)),
    );
    request.risk_budget.max_adv_pct = Some(Decimal::from_str_radix("0.01", 10).unwrap());

    let res = validate_sizing_request(&request, &budget);
    match res {
        Err(RiskError::SizingRefusal(SizingRiskRefusal::MaxAdvParticipationExceeded {
            participation_pct,
            max_adv_pct,
        })) => {
            assert_eq!(max_adv_pct, Decimal::from_str_radix("0.01", 10).unwrap());
            assert_eq!(
                participation_pct,
                Decimal::from_str_radix("0.1", 10).unwrap()
            );
            assert!(participation_pct > max_adv_pct);
        }
        other => panic!("expected MaxAdvParticipationExceeded, got {other:?}"),
    }
}

#[test]
fn max_adv_participation_within_limits_passes() {
    // Wider stop (50 points) → quantity 200 → 0.2% participation vs a 10% cap.
    let (mut request, mut budget) = make_request(
        Decimal::from(2500),
        Some(Decimal::from(2450)),
        Some(Quantity::from_u64(100_000)),
    );
    budget.max_adv_pct = Some(Decimal::from_str_radix("0.1", 10).unwrap());
    request.risk_budget = budget.clone();

    let res = validate_sizing_request(&request, &budget);
    assert!(
        res.is_ok(),
        "expected within-ADV-cap sizing to pass, got {res:?}"
    );
}

#[test]
fn portfolio_risk_cap_formats_actual_currency() {
    let usd_refusal = SizingRiskRefusal::PortfolioRiskCapExceeded {
        requested_risk_pct: Decimal::from_str_radix("0.08", 10).unwrap(),
        max_portfolio_risk_pct: Decimal::from_str_radix("0.05", 10).unwrap(),
        risk_at_stake: Money::new(Decimal::from(8_000), Currency::Usd),
        equity: Money::new(Decimal::from(100_000), Currency::Usd),
    };
    let text = format!("{usd_refusal}");
    assert!(
        text.contains("USD"),
        "expected USD in formatted refusal, got: {text}"
    );
    assert!(
        !text.contains("INR"),
        "unexpected INR in formatted refusal, got: {text}"
    );

    // The INR path still formats INR.
    let inr_refusal = SizingRiskRefusal::PortfolioRiskCapExceeded {
        requested_risk_pct: Decimal::from_str_radix("0.08", 10).unwrap(),
        max_portfolio_risk_pct: Decimal::from_str_radix("0.05", 10).unwrap(),
        risk_at_stake: Money::inr(Decimal::from(8_000)),
        equity: Money::inr(Decimal::from(100_000)),
    };
    assert!(format!("{inr_refusal}").contains("INR"));
}

#[test]
fn new_sizing_refusals_have_rules_and_display() {
    let weight = SizingRiskRefusal::MaxWeightExceeded {
        weight_pct: Decimal::ONE,
        max_weight_pct: Decimal::new(5, 1),
    };
    assert_eq!(weight.rule(), "max_weight_exceeded");
    assert!(format!("{weight}").contains("Position weight exceeded"));

    let adv = SizingRiskRefusal::MaxAdvParticipationExceeded {
        participation_pct: Decimal::ONE,
        max_adv_pct: Decimal::new(5, 1),
    };
    assert_eq!(adv.rule(), "max_adv_participation_exceeded");
    assert!(format!("{adv}").contains("ADV20 participation exceeded"));
}

#[test]
fn new_sizing_refusals_serde_roundtrip() {
    let cases = vec![
        SizingRiskRefusal::MaxWeightExceeded {
            weight_pct: Decimal::ONE,
            max_weight_pct: Decimal::new(5, 1),
        },
        SizingRiskRefusal::MaxAdvParticipationExceeded {
            participation_pct: Decimal::ONE,
            max_adv_pct: Decimal::new(5, 1),
        },
    ];
    for refusal in cases {
        let json = serde_json::to_string(&refusal).expect("serialize");
        let back: SizingRiskRefusal = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(refusal, back);
    }
}
