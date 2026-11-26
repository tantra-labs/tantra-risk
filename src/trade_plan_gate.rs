//! Stop-protection validation gate for trade plans and trailing-stop updates.
//!
//! Synchronously validates a [`TradePlan`]'s initial stop loss against the
//! mandatory-SL, SL-distance band, price-collar, and max-loss rules, and
//! enforces the no-widening rule on trailing-stop updates. Pure `Decimal`
//! comparisons only — zero I/O, zero clocks, preserving the sub-15 µs
//! synchronous latency invariant of `tantra-risk`.
//!
//! Gap 0005 — stop-protection risk validation in `tantra-risk`.

use rust_decimal::Decimal;

use tantra_core::order::OrderSide;
use tantra_core::trade_plan::TradePlan;
use tantra_core::{Money, Price};

use crate::refusal::RiskRefusal;

/// Stop-protection configuration for [`validate_trade_plan`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StopProtectionConfig {
    /// Minimum stop distance as a multiple of the scrip ATR (0.5 = half ATR).
    pub min_sl_atr: Decimal,
    /// Maximum stop distance as a fraction of the entry price (0.02 = 2%).
    pub max_sl_pct: Decimal,
    /// Maximum stop distance as a multiple of the scrip ADR% (1.0 = 1×).
    pub max_stop_adr_mult: Decimal,
    /// Per-strategy worst-case loss cap, when configured.
    pub max_loss: Option<Money>,
}

impl Default for StopProtectionConfig {
    fn default() -> Self {
        Self {
            min_sl_atr: Decimal::new(5, 1),  // 0.5
            max_sl_pct: Decimal::new(2, 2),  // 0.02
            max_stop_adr_mult: Decimal::ONE, // 1.0
            max_loss: None,
        }
    }
}

/// Validate a trade plan's initial stop protection, failing closed on every
/// stop-protection rule.
///
/// # Errors
/// Returns a [`RiskRefusal`] for:
/// 1. `MissingStopProtection` — a zero stop or a stop on the wrong side of
///    the entry for the plan's side.
/// 2. `StopTooTight` — `|entry − stop| < min_sl_atr × ATR` when `scrip_atr` is
///    supplied.
/// 3. `StopTooWide` — `|entry − stop| / entry × 100` exceeds `max_sl_pct × 100`
///    or, when `scrip_adr_pct` is supplied, exceeds
///    `max_stop_adr_mult × adr_pct`.
/// 4. `StopOutsidePriceCollar` — the stop sits outside
///    `reference ± reference × collar_bps / 10_000` when `collar_bps` is
///    supplied.
/// 5. `MaxLossExceeded` — `plan.risk_at_stake` exceeds `config.max_loss` when
///    configured.
pub fn validate_trade_plan(
    plan: &TradePlan,
    reference: Price,
    scrip_atr: Option<Decimal>,
    scrip_adr_pct: Option<Decimal>,
    collar_bps: Option<Decimal>,
    config: &StopProtectionConfig,
) -> Result<(), RiskRefusal> {
    let entry = plan.entry.value();
    let stop = plan.initial_stop.value();

    // 1. Mandatory SL: non-zero and on the correct side of the entry.
    if stop == Decimal::ZERO {
        return Err(RiskRefusal::MissingStopProtection {
            reason: "initial stop is zero; a plan must carry a protective stop loss".to_string(),
        });
    }
    let on_side = match plan.side {
        OrderSide::Buy => stop < entry,
        OrderSide::Sell => stop > entry,
    };
    if !on_side {
        return Err(RiskRefusal::MissingStopProtection {
            reason: format!(
                "initial stop {stop} is on the wrong side of entry {entry} for {:?}",
                plan.side
            ),
        });
    }

    let stop_dist = (entry - stop).abs();

    // 2a. SL band — too tight: closer to entry than min_sl_atr × ATR.
    if let Some(atr) = scrip_atr {
        let min_sl_dist = config.min_sl_atr * atr;
        if stop_dist < min_sl_dist {
            return Err(RiskRefusal::StopTooTight {
                stop_dist,
                min_sl_dist,
            });
        }
    }

    // 2b. SL band — too wide: beyond max_sl_pct of the entry or beyond
    //     max_stop_adr_mult × ADR%. Percentage comparisons only run when the
    //     entry is strictly positive (a zero entry is a degenerate plan).
    if entry > Decimal::ZERO {
        let distance_frac = stop_dist / entry;
        let max_sl_dist = config.max_sl_pct * entry;
        if distance_frac > config.max_sl_pct {
            return Err(RiskRefusal::StopTooWide {
                stop_dist,
                max_sl_dist,
            });
        }
        if let Some(adr_pct) = scrip_adr_pct {
            let max_adr_dist = config.max_stop_adr_mult * adr_pct * entry / Decimal::from(100);
            if distance_frac * Decimal::from(100) > config.max_stop_adr_mult * adr_pct {
                return Err(RiskRefusal::StopTooWide {
                    stop_dist,
                    max_sl_dist: max_adr_dist,
                });
            }
        }
    }

    // 3. SL within the price collar around the reference.
    if let Some(collar_bps) = collar_bps {
        let collar_frac = collar_bps / Decimal::from(10_000);
        let reference_val = reference.value();
        let lower = reference_val * (Decimal::ONE - collar_frac);
        let upper = reference_val * (Decimal::ONE + collar_frac);
        if stop < lower || stop > upper {
            return Err(RiskRefusal::StopOutsidePriceCollar {
                stop: plan.initial_stop,
                reference,
                collar_bps,
            });
        }
    }

    // 4. Max loss per strategy.
    if let Some(max_loss) = config.max_loss {
        if plan.risk_at_stake.amount() > max_loss.amount() {
            return Err(RiskRefusal::MaxLossExceeded {
                risk_at_stake: plan.risk_at_stake,
                max_loss,
            });
        }
    }

    Ok(())
}

/// Validate a trailing-stop update: stops may tighten, never widen.
///
/// For a long (`Buy`), the stop must move up or hold (`new >= current`); for a
/// short (`Sell`), the stop must move down or hold (`new <= current`). Any
/// request that loosens the protective stop is refused with
/// [`RiskRefusal::StopWideningDenied`].
pub fn validate_stop_update(
    side: OrderSide,
    current_stop: Price,
    new_stop: Price,
) -> Result<(), RiskRefusal> {
    let tightens = match side {
        OrderSide::Buy => new_stop.value() >= current_stop.value(),
        OrderSide::Sell => new_stop.value() <= current_stop.value(),
    };
    if tightens {
        Ok(())
    } else {
        Err(RiskRefusal::StopWideningDenied {
            current_stop,
            new_stop,
        })
    }
}
