//! Plan 0002 — Close declared-but-unenforced risk controls (gaps 0001–0004).
//!
//! TDD suite for `docs/plans/0002-close-declared-unenforced-risk-controls.md`:
//! - 0002 (High): reduce-only mode (`GlobalState::ReduceOnly`) enforced on every check path.
//! - 0001 (Med):  market-order price collar (`price_collar_bps`) enforced.
//! - 0003 (Med):  `check_order_fast` per-scrip OTR / warning / cooldown parity with `check_order`.
//! - 0004 (Low):  OTR window config knobs (`window_1min_ms` / `window_5min_ms`) honored.

use chrono::Utc;
use rust_decimal::Decimal;
use tantra_core::{
    Decision, Exchange, InstrumentId, Money, OrderRequest, OrderSide, OrderType, PositionSide,
    PositionSizingRequest, Price, PriceBand, Quantity, RiskBudget, SizingMethod,
};
use tantra_risk::{
    AlgoState, GlobalState, OtrConfig, RiskEngine, RiskError, RiskLimits, RiskRefusal,
    SizingRiskRefusal,
};

fn nse(symbol: &str) -> InstrumentId {
    InstrumentId::new(Exchange::Nse, symbol)
}

fn market_order(inst: InstrumentId, side: OrderSide, algo_id: &str) -> OrderRequest {
    OrderRequest::new(
        inst,
        side,
        OrderType::Market,
        Quantity::from_u64(10),
        algo_id,
    )
}

fn make_sizing_request(
    equity: Decimal,
    entry_p: Decimal,
    risk_pct: Decimal,
) -> (PositionSizingRequest, RiskBudget) {
    let inst = nse("RELIANCE");
    let budget = RiskBudget::new(Money::inr(equity), risk_pct, Quantity::from_u64(1));
    let req = PositionSizingRequest::new(
        inst,
        PositionSide::Long,
        Price::new(entry_p),
        Money::inr(equity),
        budget.clone(),
        SizingMethod::FixedFractional { risk_pct },
    )
    .with_stop_loss(Price::new(entry_p - Decimal::from(50)));
    (req, budget)
}

// ============================================================================
// Gap 0002 — Reduce-only mode enforcement
// ============================================================================

#[test]
fn reduce_only_refuses_new_buy_via_check_order() {
    let mut engine = RiskEngine::default();
    engine.register_algo("algo-1");
    let now = Utc::now();
    let inst = nse("INFY");

    assert!(
        engine
            .check_order(&market_order(inst.clone(), OrderSide::Buy, "algo-1"), now)
            .is_ok()
    );

    engine.set_reduce_only();
    assert!(engine.global_state == GlobalState::ReduceOnly);

    let res = engine.check_order(&market_order(inst.clone(), OrderSide::Buy, "algo-1"), now);
    assert!(matches!(res, Err(RiskRefusal::ReduceOnlyViolation { .. })));
    assert_eq!(
        res.unwrap_err().rule(),
        "reduce_only_violation",
        "refusal must carry the machine-readable rule name"
    );
}

#[test]
fn reduce_only_allows_closing_sell_via_check_order() {
    let mut engine = RiskEngine::default();
    engine.register_algo("algo-1");
    let now = Utc::now();
    let inst = nse("INFY");

    engine.set_reduce_only();
    // Sell = close a long: the documented position-free approximation.
    assert!(
        engine
            .check_order(&market_order(inst.clone(), OrderSide::Sell, "algo-1"), now)
            .is_ok()
    );
}

#[test]
fn reduce_only_enforced_on_fast_path() {
    let mut engine = RiskEngine::default();
    engine.register_algo("algo-1");
    let now = Utc::now();
    let inst = nse("INFY");

    engine.set_reduce_only();
    assert!(matches!(
        engine.check_order_fast(
            "algo-1",
            &inst,
            OrderSide::Buy,
            Quantity::from_u64(10),
            None,
            now
        ),
        Err(RiskRefusal::ReduceOnlyViolation { .. })
    ));
    // Same guarantee as the full path: a reducing Sell still passes.
    assert!(
        engine
            .check_order_fast(
                "algo-1",
                &inst,
                OrderSide::Sell,
                Quantity::from_u64(10),
                None,
                now
            )
            .is_ok()
    );
}

#[test]
fn reduce_only_enforced_on_check_decision() {
    let mut engine = RiskEngine::default();
    engine.register_algo("algo-1");
    let now = Utc::now();

    let buy = Decision::PlaceOrder {
        symbol: "INFY".to_string(),
        side: OrderSide::Buy,
        qty: Quantity::from_u64(10),
        price_limit: None,
        algo_id: "algo-1".to_string(),
        rationale: None,
    };
    let sell = Decision::PlaceOrder {
        symbol: "INFY".to_string(),
        side: OrderSide::Sell,
        qty: Quantity::from_u64(10),
        price_limit: None,
        algo_id: "algo-1".to_string(),
        rationale: None,
    };
    let modify = Decision::ModifyOrder {
        instrument: nse("INFY"),
        new_qty: Some(Quantity::from_u64(20)),
        new_price: None,
        algo_id: "algo-1".to_string(),
        rationale: None,
    };
    let cancel = Decision::CancelOrder {
        instrument: nse("INFY"),
        algo_id: Some("algo-1".to_string()),
        rationale: None,
    };

    engine.set_reduce_only();

    // Order-opening BUY and size-changing MODIFY are refused under reduce-only.
    assert!(matches!(
        engine.check_decision(&buy, now),
        Err(RiskRefusal::ReduceOnlyViolation { .. })
    ));
    assert!(matches!(
        engine.check_decision(&modify, now),
        Err(RiskRefusal::ReduceOnlyViolation { .. })
    ));
    // Closing SELL and exposure-reducing CANCEL are approved.
    assert!(engine.check_decision(&sell, now).is_ok());
    assert!(engine.check_decision(&cancel, now).is_ok());
}

#[test]
fn reduce_only_refuses_position_sizing() {
    let (req, budget) = make_sizing_request(
        Decimal::from(1_000_000),
        Decimal::from(2500),
        Decimal::from_str_radix("0.01", 10).unwrap(),
    );
    let mut engine = RiskEngine::default();
    // Baseline: sizing is approved when active.
    assert!(engine.validate_sizing_request(&req, &budget).is_ok());

    engine.set_reduce_only();
    let err = engine.validate_sizing_request(&req, &budget).unwrap_err();
    match err {
        RiskError::SizingRefusal(SizingRiskRefusal::ReduceOnlyViolation { .. }) => {}
        other => panic!("expected ReduceOnlyViolation, got {other:?}"),
    }
}

#[test]
fn resume_global_restores_full_approval() {
    let mut engine = RiskEngine::default();
    engine.register_algo("algo-1");
    let now = Utc::now();
    let inst = nse("INFY");

    engine.set_reduce_only();
    assert!(
        engine
            .check_order(&market_order(inst.clone(), OrderSide::Buy, "algo-1"), now)
            .is_err()
    );

    engine.resume_global();
    assert!(engine.global_state == GlobalState::Active);
    assert!(
        engine
            .check_order(&market_order(inst, OrderSide::Buy, "algo-1"), now)
            .is_ok()
    );
}

// ============================================================================
// Gap 0001 — Market-order price collar (`price_collar_bps`)
// ============================================================================

#[test]
fn price_collar_within_collar_passes() {
    // Band ±4% around a 2500 reference; collar 600 bps (6%) -> window [2350, 2650].
    let limits = RiskLimits {
        price_collar_bps: Some(600),
        ..Default::default()
    };
    let mut engine = RiskEngine::new(limits, OtrConfig::default());
    engine.register_algo("algo-1");
    let now = Utc::now();
    let inst = nse("RELIANCE");
    engine.set_price_band(
        inst.clone(),
        PriceBand::new(
            Price::from_rupees("2400.00").unwrap(),
            Price::from_rupees("2600.00").unwrap(),
            Price::from_rupees("2500.00").unwrap(),
        ),
    );

    assert!(
        engine
            .check_order(&market_order(inst.clone(), OrderSide::Buy, "algo-1"), now)
            .is_ok()
    );
    assert!(
        engine
            .check_order(&market_order(inst.clone(), OrderSide::Sell, "algo-1"), now)
            .is_ok()
    );
}

#[test]
fn price_collar_beyond_collar_refused() {
    // Band ±20% around a 2500 reference; collar 500 bps (5%) -> window [2375, 2625].
    let limits = RiskLimits {
        price_collar_bps: Some(500),
        ..Default::default()
    };
    let mut engine = RiskEngine::new(limits, OtrConfig::default());
    engine.register_algo("algo-1");
    let now = Utc::now();
    let inst = nse("RELIANCE");
    engine.set_price_band(
        inst.clone(),
        PriceBand::new(
            Price::from_rupees("2000.00").unwrap(),
            Price::from_rupees("3000.00").unwrap(),
            Price::from_rupees("2500.00").unwrap(),
        ),
    );

    // A market BUY could fill up at the upper band edge, beyond the collar.
    let res = engine.check_order(&market_order(inst.clone(), OrderSide::Buy, "algo-1"), now);
    assert!(matches!(res, Err(RiskRefusal::PriceCollarViolation { .. })));
    assert_eq!(res.unwrap_err().rule(), "price_collar_violation");

    // A market SELL could fill down at the lower band edge, beyond the collar.
    let res = engine.check_order(&market_order(inst.clone(), OrderSide::Sell, "algo-1"), now);
    assert!(matches!(res, Err(RiskRefusal::PriceCollarViolation { .. })));
}

#[test]
fn price_collar_leaves_limit_orders_untouched() {
    let limits = RiskLimits {
        price_collar_bps: Some(500),
        ..Default::default()
    };
    let mut engine = RiskEngine::new(limits, OtrConfig::default());
    engine.register_algo("algo-1");
    let now = Utc::now();
    let inst = nse("RELIANCE");
    engine.set_price_band(
        inst.clone(),
        PriceBand::new(
            Price::from_rupees("2000.00").unwrap(),
            Price::from_rupees("3000.00").unwrap(),
            Price::from_rupees("2500.00").unwrap(),
        ),
    );

    // A limit order inside the band passes; the collar only gates market orders.
    let limit = OrderRequest::new(
        inst.clone(),
        OrderSide::Buy,
        OrderType::Limit(Price::from_rupees("2550.00").unwrap()),
        Quantity::from_u64(10),
        "algo-1",
    );
    assert!(engine.check_order(&limit, now).is_ok());
}

#[test]
fn price_collar_skipped_without_band_or_on_fast_path_parity() {
    let limits = RiskLimits {
        price_collar_bps: Some(500),
        ..Default::default()
    };
    let mut engine = RiskEngine::new(limits, OtrConfig::default());
    engine.register_algo("algo-1");
    let now = Utc::now();
    let inst = nse("RELIANCE");

    // No band -> no reference price -> collar is not applied.
    assert!(
        engine
            .check_order(&market_order(inst.clone(), OrderSide::Buy, "algo-1"), now)
            .is_ok()
    );

    // With a band beyond the collar, the fast path enforces the same collar.
    engine.set_price_band(
        inst.clone(),
        PriceBand::new(
            Price::from_rupees("2000.00").unwrap(),
            Price::from_rupees("3000.00").unwrap(),
            Price::from_rupees("2500.00").unwrap(),
        ),
    );
    assert!(matches!(
        engine.check_order_fast(
            "algo-1",
            &inst,
            OrderSide::Buy,
            Quantity::from_u64(10),
            None,
            now
        ),
        Err(RiskRefusal::PriceCollarViolation { .. })
    ));
}

// ============================================================================
// Gap 0003 — `check_order_fast` OTR parity with `check_order`
// ============================================================================

fn tight_otr_config() -> OtrConfig {
    OtrConfig {
        max_otr: Decimal::from(3),
        min_orders_before_enforcement: 1,
        warning_threshold: Decimal::from(0),
        window_1min_ms: 60_000,
        window_5min_ms: 300_000,
        halt_cooldown_ms: 0,
    }
}

#[test]
fn fast_path_enforces_per_scrip_otr_like_full_path() {
    // Two algos on one scrip: each algo stays under max_otr, but the scrip
    // aggregate crosses it — only the per-scrip check can catch it.
    let mut full = RiskEngine::new(RiskLimits::default(), tight_otr_config());
    let mut fast = RiskEngine::new(RiskLimits::default(), tight_otr_config());
    full.register_algo("algo-a");
    full.register_algo("algo-b");
    fast.register_algo("algo-a");
    fast.register_algo("algo-b");
    let now = Utc::now();
    let inst = nse("WIPRO");

    let mut full_scrip_breach = false;
    for i in 0..4 {
        let algo = if i % 2 == 0 { "algo-a" } else { "algo-b" };
        if matches!(
            full.check_order(&market_order(inst.clone(), OrderSide::Buy, algo), now),
            Err(RiskRefusal::ScripOtrBreached { .. })
        ) {
            full_scrip_breach = true;
            break;
        }
    }

    let mut fast_scrip_breach = false;
    for i in 0..4 {
        let algo = if i % 2 == 0 { "algo-a" } else { "algo-b" };
        if matches!(
            fast.check_order_fast(
                algo,
                &inst,
                OrderSide::Buy,
                Quantity::from_u64(10),
                None,
                now
            ),
            Err(RiskRefusal::ScripOtrBreached { .. })
        ) {
            fast_scrip_breach = true;
            break;
        }
    }

    assert!(
        full_scrip_breach,
        "full path must trip the per-scrip OTR circuit breaker"
    );
    assert!(
        fast_scrip_breach,
        "fast path must trip the per-scrip OTR circuit breaker (parity with check_order)"
    );
}

#[test]
fn fast_path_auto_resumes_halted_algo_after_cooldown_like_full_path() {
    let mut full = RiskEngine::new(
        RiskLimits::default(),
        OtrConfig {
            halt_cooldown_ms: 1000,
            ..Default::default()
        },
    );
    let mut fast = full.clone();
    full.register_algo("algo-c");
    fast.register_algo("algo-c");
    let now = Utc::now();
    let inst = nse("TCS");

    full.halt_algo("algo-c", "OTR cooldown test", now);
    fast.halt_algo("algo-c", "OTR cooldown test", now);

    // Both paths reject immediately after the halt.
    assert!(matches!(
        full.check_order(&market_order(inst.clone(), OrderSide::Buy, "algo-c"), now),
        Err(RiskRefusal::AlgoHalted { .. })
    ));
    assert!(matches!(
        fast.check_order_fast(
            "algo-c",
            &inst,
            OrderSide::Buy,
            Quantity::from_u64(10),
            None,
            now
        ),
        Err(RiskRefusal::AlgoHalted { .. })
    ));

    // After the cooldown, both paths auto-resume the algo.
    let after = now + chrono::Duration::milliseconds(1001);
    assert!(
        full.check_order(&market_order(inst.clone(), OrderSide::Buy, "algo-c"), after)
            .is_ok()
    );
    assert!(
        fast.check_order_fast(
            "algo-c",
            &inst,
            OrderSide::Buy,
            Quantity::from_u64(10),
            None,
            after
        )
        .is_ok()
    );
    assert!(matches!(
        fast.algo_states.get("algo-c"),
        Some(AlgoState::Active)
    ));
}

// ============================================================================
// Gap 0004 — OTR window config knobs consulted
// ============================================================================

#[test]
fn custom_window_1min_is_enforced() {
    // window_1min_ms = 5s with max_otr 2: a 10s gap lets old events age out of
    // the *configured* window even though they are still inside the hardcoded 60s.
    let otr_config = OtrConfig {
        max_otr: Decimal::from(2),
        min_orders_before_enforcement: 1,
        warning_threshold: Decimal::from(0),
        window_1min_ms: 5_000,
        window_5min_ms: 300_000,
        halt_cooldown_ms: 0,
    };
    let mut engine = RiskEngine::new(RiskLimits::default(), otr_config);
    engine.register_algo("algo-1");
    let t0 = Utc::now();
    let inst = nse("HDFC");

    let req = market_order(inst, OrderSide::Buy, "algo-1");
    // Two orders at t0: OTR 2, not > 2 -> pass.
    assert!(engine.check_order(&req, t0).is_ok());
    assert!(engine.check_order(&req, t0).is_ok());

    // At t0+10s the t0 events have fallen out of the 5s window, so OTR resets.
    let later = t0 + chrono::Duration::seconds(10);
    assert!(engine.check_order(&req, later).is_ok());
    assert!(engine.check_order(&req, later).is_ok());

    // Contrast: with the default 60s window the same 4 orders DO breach (OTR 3 > 2).
    let mut fixed = RiskEngine::new(
        RiskLimits::default(),
        OtrConfig {
            max_otr: Decimal::from(2),
            min_orders_before_enforcement: 1,
            ..Default::default()
        },
    );
    fixed.register_algo("algo-1");
    let req = market_order(nse("HDFC"), OrderSide::Buy, "algo-1");
    assert!(fixed.check_order(&req, t0).is_ok());
    assert!(fixed.check_order(&req, t0).is_ok());
    let third = fixed.check_order(&req, later);
    assert!(
        matches!(third, Err(RiskRefusal::OtrBreached { .. })),
        "with a 60s window the third order must breach OTR, proving the knob has an effect"
    );
}

#[test]
fn window_otr_5min_honors_configured_window() {
    use tantra_risk::OtrTracker;
    let config = OtrConfig {
        window_5min_ms: 10_000,
        ..Default::default()
    };
    let mut tracker = OtrTracker::new();
    let base = Utc::now();
    for _ in 0..3 {
        tracker.record_order_placement(base);
    }
    tracker.record_fill(base);

    // 5s in: events still inside the configured 10s window -> OTR 3/1 = 3.
    let t5 = base + chrono::Duration::seconds(5);
    assert_eq!(tracker.window_otr_5min(t5, &config), Decimal::from(3));

    // 15s in: events have aged out of the configured 10s window -> OTR 0.
    let t15 = base + chrono::Duration::seconds(15);
    assert_eq!(tracker.window_otr_5min(t15, &config), Decimal::ZERO);
}

#[test]
fn window_otr_1min_honors_configured_window() {
    use tantra_risk::OtrTracker;
    let config = OtrConfig {
        window_1min_ms: 10_000,
        ..Default::default()
    };
    let mut tracker = OtrTracker::new();
    let base = Utc::now();
    for _ in 0..3 {
        tracker.record_order_placement(base);
    }
    tracker.record_fill(base);

    let t5 = base + chrono::Duration::seconds(5);
    assert_eq!(tracker.window_otr_1min(t5, &config), Decimal::from(3));

    let t15 = base + chrono::Duration::seconds(15);
    assert_eq!(tracker.window_otr_1min(t15, &config), Decimal::ZERO);
}
