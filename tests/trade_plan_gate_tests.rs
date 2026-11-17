//! Integration tests for the trade-plan stop-protection gate (gap 0005).
//!
//! Covers each new `RiskRefusal` (missing SL, too-tight, too-wide, outside
//! collar, max-loss), the no-widening TSL rule, the engine wiring, and serde
//! round-tripping of the new refusal variants.

use rust_decimal::Decimal;
use tantra_core::order::OrderSide;
use tantra_core::trade_plan::{TradePlan, TrailRule, TrailSpec};
use tantra_core::{Exchange, InstrumentId, Money, Price, Quantity};
use tantra_risk::refusal::RiskRefusal;
use tantra_risk::trade_plan_gate::{
    StopProtectionConfig, validate_stop_update, validate_trade_plan,
};
use tantra_risk::{OtrConfig, RiskEngine, RiskLimits};

/// Build a bare trade plan with the given entry/stop.
fn make_plan(side: OrderSide, entry: u64, stop: u64) -> TradePlan {
    let entry_d = Decimal::from(entry);
    let stop_d = Decimal::from(stop);
    TradePlan {
        instrument: InstrumentId::new(Exchange::Nse, "RELIANCE"),
        side,
        entry: Price::new(entry_d),
        initial_stop: Price::new(stop_d),
        risk_per_unit: (entry_d - stop_d).abs(),
        quantity: Quantity::from_u64(1),
        risk_at_stake: Money::inr((entry_d - stop_d).abs()),
        targets: Vec::new(),
        trail: TrailSpec {
            rule: TrailRule::Off,
            activation_after_r: Decimal::ZERO,
            breakeven_after: None,
            time_stop: None,
        },
        reward_risk: Decimal::ZERO,
        spec_version: 1,
        rationale: "test plan".to_string(),
    }
}

#[test]
fn validate_trade_plan_accepts_a_valid_plan() {
    // 2500 → 2450 is a 2% stop (within the 2% band), ATR 100 → min dist 50.
    let plan = make_plan(OrderSide::Buy, 2500, 2450);
    let res = validate_trade_plan(
        &plan,
        Price::new(Decimal::from(2500)),
        Some(Decimal::from(100)),
        None,
        None,
        &StopProtectionConfig::default(),
    );
    assert!(res.is_ok(), "expected valid plan to pass, got {res:?}");
}

#[test]
fn missing_stop_protection_fires_for_zero_or_wrong_side_stop() {
    let config = StopProtectionConfig::default();
    let reference = Price::new(Decimal::from(2500));

    // Zero stop.
    let zero_stop = make_plan(OrderSide::Buy, 2500, 0);
    let res = validate_trade_plan(&zero_stop, reference, None, None, None, &config);
    assert!(matches!(
        res,
        Err(RiskRefusal::MissingStopProtection { .. })
    ));

    // Buy with stop above entry.
    let wrong_buy = make_plan(OrderSide::Buy, 2500, 2550);
    let res = validate_trade_plan(&wrong_buy, reference, None, None, None, &config);
    assert!(matches!(
        res,
        Err(RiskRefusal::MissingStopProtection { .. })
    ));

    // Sell with stop below entry.
    let wrong_sell = make_plan(OrderSide::Sell, 2500, 2450);
    let res = validate_trade_plan(&wrong_sell, reference, None, None, None, &config);
    assert!(matches!(
        res,
        Err(RiskRefusal::MissingStopProtection { .. })
    ));
}

#[test]
fn stop_too_tight_fires_when_below_min_sl_atr() {
    // Stop 2495 → dist 5, ATR 100 → min_sl_dist 50.
    let plan = make_plan(OrderSide::Buy, 2500, 2495);
    let res = validate_trade_plan(
        &plan,
        Price::new(Decimal::from(2500)),
        Some(Decimal::from(100)),
        None,
        None,
        &StopProtectionConfig::default(),
    );
    match res {
        Err(RiskRefusal::StopTooTight {
            stop_dist,
            min_sl_dist,
        }) => {
            assert_eq!(stop_dist, Decimal::from(5));
            assert_eq!(min_sl_dist, Decimal::from(50));
        }
        other => panic!("expected StopTooTight, got {other:?}"),
    }
}

#[test]
fn stop_too_wide_fires_for_max_sl_pct_and_adr() {
    let config = StopProtectionConfig::default();
    let reference = Price::new(Decimal::from(2500));

    // 2500 → 2300: dist 200 = 8% of entry > 2% → too wide.
    let wide_plan = make_plan(OrderSide::Buy, 2500, 2300);
    let res = validate_trade_plan(&wide_plan, reference, None, None, None, &config);
    match res {
        Err(RiskRefusal::StopTooWide {
            stop_dist,
            max_sl_dist,
        }) => {
            assert_eq!(stop_dist, Decimal::from(200));
            assert_eq!(max_sl_dist, Decimal::new(50, 0)); // 0.02 × 2500
        }
        other => panic!("expected StopTooWide, got {other:?}"),
    }

    // ADR guard: entry 2000 → stop 1970 = 1.5% (within the 2% max_sl_pct band)
    // but beyond mult 1.0 × adr_pct 1.0 = 1% → max_adr_dist = 20; dist 30 > 20.
    let adr_wide_plan = make_plan(OrderSide::Buy, 2000, 1970);
    let res = validate_trade_plan(
        &adr_wide_plan,
        Price::new(Decimal::from(2000)),
        None,
        Some(Decimal::ONE),
        None,
        &config,
    );
    match res {
        Err(RiskRefusal::StopTooWide {
            stop_dist,
            max_sl_dist,
        }) => {
            assert_eq!(stop_dist, Decimal::from(30));
            assert_eq!(max_sl_dist, Decimal::from(20)); // 1.0 × 1.0 × 2000 / 100
        }
        other => panic!("expected StopTooWide (ADR), got {other:?}"),
    }
}

#[test]
fn stop_outside_price_collar_fires() {
    let config = StopProtectionConfig::default();
    let reference = Price::new(Decimal::from(2500));
    let collar_bps = Some(Decimal::from(100)); // ±1% → [2475, 2525]

    // Stop 2460 sits 40 points below entry and 15 points below the lower band.
    let outside = make_plan(OrderSide::Buy, 2500, 2460);
    let res = validate_trade_plan(&outside, reference, None, None, collar_bps, &config);
    match res {
        Err(RiskRefusal::StopOutsidePriceCollar {
            stop,
            reference: ref_price,
            collar_bps: bps,
        }) => {
            assert_eq!(stop.value(), Decimal::from(2460));
            assert_eq!(ref_price, reference);
            assert_eq!(bps, Decimal::from(100));
        }
        other => panic!("expected StopOutsidePriceCollar, got {other:?}"),
    }

    // A stop inside the collar passes.
    let inside = make_plan(OrderSide::Buy, 2500, 2480);
    let res = validate_trade_plan(&inside, reference, None, None, collar_bps, &config);
    assert!(res.is_ok(), "expected in-collar stop to pass, got {res:?}");
}

#[test]
fn max_loss_exceeded_fires_when_risk_breaches_strategy_budget() {
    let reference = Price::new(Decimal::from(2500));
    let mut plan = make_plan(OrderSide::Buy, 2500, 2450);
    plan.risk_at_stake = Money::inr(Decimal::from(1000));

    let config = StopProtectionConfig {
        max_loss: Some(Money::inr(Decimal::from(500))),
        ..StopProtectionConfig::default()
    };

    let res = validate_trade_plan(&plan, reference, None, None, None, &config);
    match res {
        Err(RiskRefusal::MaxLossExceeded {
            risk_at_stake,
            max_loss,
        }) => {
            assert_eq!(risk_at_stake, Money::inr(Decimal::from(1000)));
            assert_eq!(max_loss, Money::inr(Decimal::from(500)));
        }
        other => panic!("expected MaxLossExceeded, got {other:?}"),
    }

    // Within the max-loss budget passes.
    plan.risk_at_stake = Money::inr(Decimal::from(500));
    let res = validate_trade_plan(&plan, reference, None, None, None, &config);
    assert!(
        res.is_ok(),
        "expected within-max-loss plan to pass, got {res:?}"
    );
}

#[test]
fn stop_update_long_tightens_upward_only() {
    let current = Price::new(Decimal::from(100));
    // Tightening (or holding) is allowed for a long.
    assert!(validate_stop_update(OrderSide::Buy, current, Price::new(Decimal::from(105))).is_ok());
    assert!(validate_stop_update(OrderSide::Buy, current, current).is_ok());

    // Widening (moving the stop down) is refused.
    let res = validate_stop_update(OrderSide::Buy, current, Price::new(Decimal::from(95)));
    match res {
        Err(RiskRefusal::StopWideningDenied {
            current_stop,
            new_stop,
        }) => {
            assert_eq!(current_stop, current);
            assert_eq!(new_stop, Price::new(Decimal::from(95)));
        }
        other => panic!("expected StopWideningDenied, got {other:?}"),
    }
}

#[test]
fn stop_update_short_tightens_downward_only() {
    let current = Price::new(Decimal::from(105));
    // Tightening (or holding) is allowed for a short.
    assert!(validate_stop_update(OrderSide::Sell, current, Price::new(Decimal::from(100))).is_ok());
    assert!(validate_stop_update(OrderSide::Sell, current, current).is_ok());

    // Widening (moving the stop up) is refused.
    let res = validate_stop_update(OrderSide::Sell, current, Price::new(Decimal::from(110)));
    assert!(matches!(res, Err(RiskRefusal::StopWideningDenied { .. })));
}

#[test]
fn engine_check_trade_plan_reuses_price_collar() {
    let limits = RiskLimits {
        price_collar_bps: Some(100),
        ..Default::default()
    };
    let mut engine = RiskEngine::new(limits, OtrConfig::default());

    let outside = make_plan(OrderSide::Buy, 2500, 2460);
    let res = engine.check_trade_plan(
        &outside,
        Price::new(Decimal::from(2500)),
        chrono::Utc::now(),
    );
    assert!(matches!(
        res,
        Err(RiskRefusal::StopOutsidePriceCollar { .. })
    ));

    let inside = make_plan(OrderSide::Buy, 2500, 2480);
    let res = engine.check_trade_plan(&inside, Price::new(Decimal::from(2500)), chrono::Utc::now());
    assert!(res.is_ok(), "expected in-collar plan to pass, got {res:?}");
}

#[test]
fn engine_check_stop_update_refuses_widening() {
    let mut engine = RiskEngine::default();
    let res = engine.check_stop_update(
        OrderSide::Buy,
        Price::new(Decimal::from(100)),
        Price::new(Decimal::from(90)),
    );
    assert!(matches!(res, Err(RiskRefusal::StopWideningDenied { .. })));

    assert!(
        engine
            .check_stop_update(
                OrderSide::Buy,
                Price::new(Decimal::from(100)),
                Price::new(Decimal::from(110)),
            )
            .is_ok()
    );
}

#[test]
fn engine_builder_and_getter_for_stop_protection() {
    let engine = RiskEngine::default().with_stop_protection(StopProtectionConfig::default());
    assert_eq!(engine.stop_protection(), &StopProtectionConfig::default());
    assert_eq!(engine.stop_protection().min_sl_atr, Decimal::new(5, 1));
    assert_eq!(engine.stop_protection().max_sl_pct, Decimal::new(2, 2));
    assert_eq!(engine.stop_protection().max_stop_adr_mult, Decimal::ONE);
    assert_eq!(engine.stop_protection().max_loss, None);
}

#[test]
fn new_refusals_serde_roundtrip() {
    let cases = vec![
        RiskRefusal::MissingStopProtection {
            reason: "no stop".to_string(),
        },
        RiskRefusal::StopTooTight {
            stop_dist: Decimal::from(1),
            min_sl_dist: Decimal::from(2),
        },
        RiskRefusal::StopTooWide {
            stop_dist: Decimal::from(3),
            max_sl_dist: Decimal::from(2),
        },
        RiskRefusal::StopOutsidePriceCollar {
            stop: Price::new(Decimal::from(100)),
            reference: Price::new(Decimal::from(101)),
            collar_bps: Decimal::from(100),
        },
        RiskRefusal::MaxLossExceeded {
            risk_at_stake: Money::inr(Decimal::from(100)),
            max_loss: Money::inr(Decimal::from(50)),
        },
        RiskRefusal::StopWideningDenied {
            current_stop: Price::new(Decimal::from(100)),
            new_stop: Price::new(Decimal::from(90)),
        },
    ];
    for refusal in cases {
        let json = serde_json::to_string(&refusal).expect("serialize");
        let back: RiskRefusal = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(refusal, back);
    }
}
