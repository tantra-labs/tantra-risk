//! Gap 0006 — missing reference data fails closed (scoped), with an explicit
//! `Permissive` opt-out restoring the legacy skip-when-missing behaviour.

use chrono::{DateTime, Duration, Utc};
use proptest::prelude::*;
use rust_decimal::Decimal;
use tantra_core::{
    Exchange, InstrumentId, OrderRequest, OrderSide, OrderType, Price, PriceBand, Quantity,
};
use tantra_risk::{OtrConfig, ReferenceField, RiskEngine, RiskLimits, RiskRefusal, RiskStrictness};

const ALGO: &str = "algo-1";

fn inst(ex: Exchange, sym: &str) -> InstrumentId {
    InstrumentId::try_new(ex, sym).unwrap()
}

fn p(s: &str) -> Price {
    Price::from_rupees(s).unwrap()
}

fn band(reference: &str) -> PriceBand {
    PriceBand::try_new(p("0.05"), p("10000.00"), p(reference)).unwrap()
}

fn engine(limits: RiskLimits) -> RiskEngine {
    let mut e = RiskEngine::new(limits, OtrConfig::default());
    e.register_algo(ALGO);
    e
}

fn permissive(mut l: RiskLimits) -> RiskLimits {
    l.strictness = RiskStrictness::Permissive;
    l
}

/// Run an order through both pipelines on fresh engines built by `mk`.
fn both(
    mk: impl Fn() -> RiskEngine,
    i: &InstrumentId,
    qty: u64,
    limit: Option<Price>,
    now: DateTime<Utc>,
) -> [Result<(), RiskRefusal>; 2] {
    let order_type = limit.map_or(OrderType::Market, OrderType::Limit);
    let req = OrderRequest::new(
        i.clone(),
        OrderSide::Buy,
        order_type,
        Quantity::from_u64(qty),
        ALGO,
    );
    let slow = mk().check_order(&req, now);
    let fast = mk().check_order_fast(ALGO, i, OrderSide::Buy, Quantity::from_u64(qty), limit, now);
    [slow, fast]
}

fn missing(i: &InstrumentId, field: ReferenceField) -> RiskRefusal {
    RiskRefusal::MissingReferenceData {
        instrument: Some(i.clone()),
        field,
    }
}

#[test]
fn collar_without_band() {
    let i = inst(Exchange::Nse, "INFY");
    let l = RiskLimits {
        price_collar_bps: Some(500),
        ..Default::default()
    };
    for r in both(|| engine(l.clone()), &i, 1, None, Utc::now()) {
        assert_eq!(r, Err(missing(&i, ReferenceField::PriceBand)));
    }
    let pl = permissive(l);
    for r in both(|| engine(pl.clone()), &i, 1, None, Utc::now()) {
        assert!(r.is_ok());
    }
}

#[test]
fn collar_with_zero_reference() {
    let i = inst(Exchange::Nse, "INFY");
    let l = RiskLimits {
        price_collar_bps: Some(500),
        ..Default::default()
    };
    let mk = |l: RiskLimits| {
        let i = i.clone();
        move || {
            let mut e = engine(l.clone());
            e.set_price_band(
                i.clone(),
                PriceBand::try_new(Price::ZERO, p("100.00"), Price::ZERO).unwrap(),
            );
            e
        }
    };
    for r in both(
        mk(l.clone()),
        &inst(Exchange::Nse, "INFY"),
        1,
        None,
        Utc::now(),
    ) {
        assert_eq!(
            r,
            Err(missing(
                &inst(Exchange::Nse, "INFY"),
                ReferenceField::ReferencePrice
            ))
        );
    }
    for r in both(
        mk(permissive(l)),
        &inst(Exchange::Nse, "INFY"),
        1,
        None,
        Utc::now(),
    ) {
        assert!(r.is_ok());
    }
}

#[test]
fn freeze_limit_missing_for_derivatives() {
    for ex in [Exchange::Nfo, Exchange::Mcx, Exchange::Cds] {
        let i = inst(ex, "FOO");
        // Empty map and non-empty map (other instrument) both refuse.
        let other = inst(Exchange::Nfo, "OTHER");
        let mut l = RiskLimits::default();
        for r in both(|| engine(l.clone()), &i, 1, Some(p("10.00")), Utc::now()) {
            assert_eq!(r, Err(missing(&i, ReferenceField::FreezeLimit)));
        }
        l.freeze_limits.insert(other, Quantity::from_u64(100));
        for r in both(|| engine(l.clone()), &i, 1, Some(p("10.00")), Utc::now()) {
            assert_eq!(r, Err(missing(&i, ReferenceField::FreezeLimit)));
        }
        let pl = permissive(l.clone());
        for r in both(|| engine(pl.clone()), &i, 1, Some(p("10.00")), Utc::now()) {
            assert!(r.is_ok());
        }
        // With an entry it passes.
        l.freeze_limits.insert(i.clone(), Quantity::from_u64(100));
        for r in both(|| engine(l.clone()), &i, 1, Some(p("10.00")), Utc::now()) {
            assert!(r.is_ok());
        }
    }
}

#[test]
fn cash_equity_needs_no_freeze_limit() {
    let i = inst(Exchange::Nse, "INFY");
    for r in both(
        || engine(RiskLimits::default()),
        &i,
        1,
        Some(p("10.00")),
        Utc::now(),
    ) {
        assert!(r.is_ok());
    }
}

#[test]
fn notional_market_order_without_band() {
    let i = inst(Exchange::Nse, "INFY");
    let l = RiskLimits {
        max_notional_per_order: Some(Decimal::from(1_000)),
        ..Default::default()
    };
    for r in both(|| engine(l.clone()), &i, 1, None, Utc::now()) {
        assert_eq!(r, Err(missing(&i, ReferenceField::ReferencePrice)));
    }
    let pl = permissive(l.clone());
    for r in both(|| engine(pl.clone()), &i, 1, None, Utc::now()) {
        assert!(r.is_ok());
    }
    // Limit orders carry their own price: still pass.
    for r in both(|| engine(l.clone()), &i, 1, Some(p("10.00")), Utc::now()) {
        assert!(r.is_ok());
    }
}

#[test]
fn limit_order_without_band_passes() {
    let i = inst(Exchange::Nse, "INFY");
    let l = RiskLimits {
        price_collar_bps: Some(500),
        ..Default::default()
    };
    for r in both(|| engine(l.clone()), &i, 1, Some(p("10.00")), Utc::now()) {
        assert!(r.is_ok());
    }
}

#[test]
fn staleness() {
    let i = inst(Exchange::Nse, "INFY");
    let now = Utc::now();
    let l = RiskLimits {
        max_reference_age_ms: Some(1_000),
        ..Default::default()
    };
    let with = |b: PriceBand, l: RiskLimits| {
        let i = i.clone();
        move || {
            let mut e = engine(l.clone());
            e.set_price_band(i.clone(), b);
            e
        }
    };
    let stale = |as_of: Option<DateTime<Utc>>| RiskRefusal::StaleReferenceData {
        instrument: i.clone(),
        as_of,
        max_age_ms: 1_000,
    };

    // No as_of -> stale.
    let b = band("100.00");
    for r in both(with(b, l.clone()), &i, 1, Some(p("100.00")), now) {
        assert_eq!(r, Err(stale(None)));
    }
    // Too old -> stale.
    let old = now - Duration::milliseconds(1_001);
    for r in both(
        with(b.with_as_of(old), l.clone()),
        &i,
        1,
        Some(p("100.00")),
        now,
    ) {
        assert_eq!(r, Err(stale(Some(old))));
    }
    // Within age -> ok; exactly at max -> ok.
    for age in [0, 500, 1_000] {
        let t = now - Duration::milliseconds(age);
        for r in both(
            with(b.with_as_of(t), l.clone()),
            &i,
            1,
            Some(p("100.00")),
            now,
        ) {
            assert!(r.is_ok());
        }
    }
    // Future as_of -> not stale.
    let fut = now + Duration::seconds(60);
    for r in both(
        with(b.with_as_of(fut), l.clone()),
        &i,
        1,
        Some(p("100.00")),
        now,
    ) {
        assert!(r.is_ok());
    }
    // Permissive ignores staleness.
    for r in both(with(b, permissive(l)), &i, 1, Some(p("100.00")), now) {
        assert!(r.is_ok());
    }
}

#[test]
fn unconfigured_refuses_everything() {
    let i = inst(Exchange::Nse, "INFY");
    let req = OrderRequest::new(
        i.clone(),
        OrderSide::Sell,
        OrderType::Limit(p("10.00")),
        Quantity::from_u64(1),
        ALGO,
    );
    let mut e = RiskEngine::unconfigured();
    e.register_algo(ALGO);
    assert!(matches!(
        e.check_order(&req, Utc::now()),
        Err(RiskRefusal::GlobalHalted { .. })
    ));
    assert!(matches!(
        e.check_order_fast(
            ALGO,
            &i,
            OrderSide::Sell,
            Quantity::from_u64(1),
            Some(p("10.00")),
            Utc::now()
        ),
        Err(RiskRefusal::GlobalHalted { .. })
    ));
}

#[test]
fn equity_missing_refusals_carry_field() {
    let e = engine(RiskLimits {
        max_position_concentration_pct: Some(Decimal::from(10)),
        max_mis_leverage: Some(Decimal::from(5)),
        ..Default::default()
    });
    let i = inst(Exchange::Nse, "INFY");
    assert_eq!(
        e.check_position_concentration(&i, Decimal::ONE, Decimal::ZERO),
        Err(missing(&i, ReferenceField::TotalEquity))
    );
    assert_eq!(
        e.check_mis_leverage(Decimal::ONE, Decimal::ZERO),
        Err(RiskRefusal::MissingReferenceData {
            instrument: None,
            field: ReferenceField::TotalEquity
        })
    );
}

#[test]
fn serde_strictness() {
    let l: RiskLimits = serde_json::from_str("{}").unwrap();
    assert_eq!(l.strictness, RiskStrictness::FailClosed);

    let perm = permissive(RiskLimits::default());
    let json = serde_json::to_string(&perm).unwrap();
    assert!(json.contains("\"permissive\""));
    assert_eq!(serde_json::from_str::<RiskLimits>(&json).unwrap(), perm);

    let json = serde_json::to_string(&RiskLimits::default()).unwrap();
    assert!(!json.contains("strictness"));
}

#[test]
fn zero_max_reference_age_invalid() {
    let l = RiskLimits {
        max_reference_age_ms: Some(0),
        ..Default::default()
    };
    assert!(!l.is_valid());
    let l = RiskLimits {
        max_reference_age_ms: Some(1),
        ..Default::default()
    };
    assert!(l.is_valid());
}

proptest! {
    #[test]
    fn derivatives_without_freeze_always_refused(
        ex in prop_oneof![Just(Exchange::Nfo), Just(Exchange::Mcx), Just(Exchange::Cds)],
        sym in "[A-Z]{2,8}",
        qty in 1u64..1_000_000,
    ) {
        let i = inst(ex, &sym);
        for r in both(|| engine(RiskLimits::default()), &i, qty, Some(p("10.00")), Utc::now()) {
            prop_assert_eq!(r, Err(missing(&i, ReferenceField::FreezeLimit)));
        }
    }

    #[test]
    fn market_order_with_collar_and_no_band_refused(
        sym in "[A-Z]{2,8}",
        qty in 1u64..1_000_000,
        bps in 1u32..10_000,
    ) {
        let i = inst(Exchange::Nse, &sym);
        let l = RiskLimits { price_collar_bps: Some(bps), ..Default::default() };
        for r in both(|| engine(l.clone()), &i, qty, None, Utc::now()) {
            prop_assert_eq!(r, Err(missing(&i, ReferenceField::PriceBand)));
        }
    }
}
