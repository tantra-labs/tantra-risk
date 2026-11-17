//! Property-based invariants for the trade-plan stop-protection gate (gap 0005).
//!
//! - The SL band check never accepts a stop closer to the entry than
//!   `min_sl_atr × ATR`.
//! - The price-collar check never accepts a stop outside `reference ± collar`.

use proptest::prelude::*;
use rust_decimal::Decimal;
use tantra_core::order::OrderSide;
use tantra_core::trade_plan::{TradePlan, TrailRule, TrailSpec};
use tantra_core::{Exchange, InstrumentId, Money, Price, Quantity};
use tantra_risk::refusal::RiskRefusal;
use tantra_risk::trade_plan_gate::{StopProtectionConfig, validate_trade_plan};

/// Build a bare trade plan with the given entry/stop (Decimal-valued).
fn make_plan_dec(side: OrderSide, entry: Decimal, stop: Decimal) -> TradePlan {
    TradePlan {
        instrument: InstrumentId::new(Exchange::Nse, "RELIANCE"),
        side,
        entry: Price::new(entry),
        initial_stop: Price::new(stop),
        risk_per_unit: (entry - stop).abs(),
        quantity: Quantity::from_u64(1),
        risk_at_stake: Money::inr((entry - stop).abs()),
        targets: Vec::new(),
        trail: TrailSpec {
            rule: TrailRule::Off,
            activation_after_r: Decimal::ZERO,
            breakeven_after: None,
            time_stop: None,
        },
        reward_risk: Decimal::ZERO,
        spec_version: 1,
        rationale: "proptest plan".to_string(),
    }
}

fn arb_side() -> impl Strategy<Value = OrderSide> {
    prop_oneof![Just(OrderSide::Buy), Just(OrderSide::Sell)]
}

// Property: the SL band check never accepts a stop with
// `stop_dist < min_sl_atr × atr` (default min_sl_atr = 0.5).
proptest! {
    #[test]
    fn never_accepts_stop_tighter_than_min_sl_atr(
        entry in 1000u64..50_000u64,
        dist in 1u64..200u64,
        atr in 1u64..400u64,
        side in arb_side(),
    ) {
        let min_sl_dist = Decimal::from(atr) / Decimal::from(2);
        // Only assert the tight cases: dist below min_sl_atr × ATR.
        if Decimal::from(dist) >= min_sl_dist {
            return Ok(());
        }
        let stop = match side {
            OrderSide::Buy => entry - dist,
            OrderSide::Sell => entry + dist,
        };
        let plan = make_plan_dec(side, Decimal::from(entry), Decimal::from(stop));
        let reference = Price::new(Decimal::from(entry));
        let res = validate_trade_plan(
            &plan,
            reference,
            Some(Decimal::from(atr)),
            None,
            None,
            &StopProtectionConfig::default(),
        );
        let too_tight = matches!(res, Err(RiskRefusal::StopTooTight { .. }));
        prop_assert!(too_tight);
    }
}

// Property: the price-collar check never accepts a stop outside
// `reference ± reference × collar_bps / 10_000`. Generators are bounded so the
// stop is within the 2% max_sl_pct band, isolating the collar refusal.
proptest! {
    #[test]
    fn never_accepts_stop_outside_price_collar(
        reference in 1000u64..10_000u64,
        collar_bps in 1u32..50u32,
        delta in 1u64..10u64,
        side in arb_side(),
    ) {
        let collar_frac = Decimal::from(collar_bps) / Decimal::from(10_000);
        let ref_dec = Decimal::from(reference);
        let lower = ref_dec * (Decimal::ONE - collar_frac);
        let upper = ref_dec * (Decimal::ONE + collar_frac);
        let (entry, stop) = match side {
            OrderSide::Buy => (ref_dec, lower - Decimal::from(delta)),
            OrderSide::Sell => (ref_dec, upper + Decimal::from(delta)),
        };
        // Sanity: the stop stays strictly positive and on the correct side.
        prop_assert!(stop > Decimal::ZERO);
        let plan = make_plan_dec(side, entry, stop);
        let res = validate_trade_plan(
            &plan,
            Price::new(ref_dec),
            None,
            None,
            Some(Decimal::from(collar_bps)),
            &StopProtectionConfig::default(),
        );
        let outside_collar = matches!(res, Err(RiskRefusal::StopOutsidePriceCollar { .. }));
        prop_assert!(outside_collar);
    }
}
