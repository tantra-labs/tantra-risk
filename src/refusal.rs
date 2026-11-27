//! Refusal types indicating why an order or decision was rejected by the risk gate.

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;
use tantra_core::{InstrumentId, Money, Price, PriceBand, Quantity};

/// Typed refusal reason from the risk gate.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "reason", content = "details", rename_all = "snake_case")]
pub enum RiskRefusal {
    /// Platform is under global halt (kill switch active).
    GlobalHalted {
        /// Explanation for the platform halt
        reason: String,
    },

    /// The specific algorithm ID is halted.
    AlgoHalted {
        /// Algo ID
        algo_id: String,
        /// Reason for halting
        reason: String,
    },

    /// Algo ID is not registered with the broker/exchange (SEBI Retail Algo Circular).
    AlgoNotRegistered {
        /// Unregistered algo ID
        algo_id: String,
        /// Reason for rejection
        reason: String,
    },

    /// Market-wide index circuit breaker triggered (10%, 15%, 20%).
    MarketCircuitHalt {
        /// Index symbol (e.g. "NIFTY 50", "SENSEX")
        index_symbol: String,
        /// Circuit percentage stage (10, 15, or 20)
        stage_pct: u8,
        /// Expected cooling-off resume time if applicable
        resume_at: Option<DateTime<Utc>>,
    },

    /// SEBI Order-to-Trade Ratio limit exceeded per scrip.
    ScripOtrBreached {
        /// Instrument with breached OTR
        instrument: InstrumentId,
        /// Current calculated OTR for this scrip
        current_otr: Decimal,
        /// Configured ceiling
        max_otr: Decimal,
    },

    /// SEBI Order-to-Trade Ratio limit exceeded.
    OtrBreached {
        /// Breaching algo ID
        algo_id: String,
        /// Current calculated OTR
        current_otr: Decimal,
        /// Configured ceiling
        max_otr: Decimal,
    },

    /// Order price violates the scrip-wise circuit price band.
    PriceBandViolation {
        /// Order price requested
        price: Price,
        /// Scrip price band bounds
        band: PriceBand,
    },

    /// Market order's worst-case fill price exceeds the configured price collar
    /// around the band reference price (`RiskLimits::price_collar_bps`).
    PriceCollarViolation {
        /// Instrument being traded
        instrument: InstrumentId,
        /// Worst-case band-edge fill price that triggered the collar
        implied_price: Price,
        /// Band reference price the collar is anchored to
        reference_price: Price,
        /// Configured collar in basis points (1 bp = 0.01%)
        collar_bps: u32,
    },

    /// Single order notional value exceeds configured limit.
    MaxNotionalExceeded {
        /// Requested order notional
        requested: Decimal,
        /// Configured limit
        limit: Decimal,
    },

    /// Order quantity exceeds exchange freeze limit for the scrip.
    FreezeLimitExceeded {
        /// Instrument being traded
        instrument: InstrumentId,
        /// Requested quantity
        requested: Quantity,
        /// Exchange freeze limit for this scrip
        limit: Quantity,
    },

    /// Order quantity exceeds configured limit.
    MaxQuantityExceeded {
        /// Requested quantity
        requested: Quantity,
        /// Configured limit
        limit: Quantity,
    },

    /// Order rate limit exceeded in the current event window.
    OrderRateExceeded {
        /// Number of orders approved in window
        count: u32,
        /// Maximum allowed
        limit: u32,
        /// Window length in milliseconds
        window_ms: u64,
    },

    /// Cumulative daily loss threshold exceeded.
    DailyLossExceeded {
        /// Realized daily loss
        current_loss: Decimal,
        /// Maximum allowable loss
        limit: Decimal,
    },

    /// Platform is in reduce-only mode and order increases risk.
    ReduceOnlyViolation {
        /// Details
        details: String,
    },

    /// Single-scrip position concentration exceeds configured percentage of portfolio equity.
    PositionConcentrationExceeded {
        /// Instrument being traded
        instrument: InstrumentId,
        /// Current position value in INR
        position_value: Decimal,
        /// Total portfolio equity in INR
        total_equity: Decimal,
        /// Maximum allowed concentration percentage
        max_pct: Decimal,
    },

    /// Gross leverage for MIS (intraday) products exceeds configured limit.
    MisLeverageExceeded {
        /// Gross exposure in INR
        gross_exposure: Decimal,
        /// Total portfolio equity in INR
        total_equity: Decimal,
        /// Maximum allowed leverage
        max_leverage: Decimal,
    },

    /// A trade plan carries no initial stop, or a stop on the wrong side of the
    /// entry (mandatory stop-protection rule).
    MissingStopProtection {
        /// Explanation of why the stop protection is missing/invalid
        reason: String,
    },

    /// Initial stop is closer to the entry than `min_sl_atr × ATR` (too tight to
    /// be a credible protective stop).
    StopTooTight {
        /// Entry-to-stop distance
        stop_dist: Decimal,
        /// Minimum permitted stop distance
        min_sl_dist: Decimal,
    },

    /// Initial stop is farther from the entry than the maximum stop distance
    /// band (`max_sl_pct` of entry or `max_stop_adr_mult × ADR%`).
    StopTooWide {
        /// Entry-to-stop distance
        stop_dist: Decimal,
        /// Maximum permitted stop distance
        max_sl_dist: Decimal,
    },

    /// Initial stop sits outside the reference ± price-collar band.
    StopOutsidePriceCollar {
        /// Stop price
        stop: Price,
        /// Band reference price the collar is anchored to
        reference: Price,
        /// Configured collar in basis points (1 bp = 0.01%)
        collar_bps: Decimal,
    },

    /// Planned worst-case loss breaches the strategy's max-loss risk budget.
    MaxLossExceeded {
        /// Planned capital at risk
        risk_at_stake: Money,
        /// Configured maximum loss
        max_loss: Money,
    },

    /// A trailing-stop update would widen the stop (loosen the protective stop).
    StopWideningDenied {
        /// Current stop price
        current_stop: Price,
        /// Requested new stop price
        new_stop: Price,
    },

    /// Order failed structural or tick validation.
    InvalidOrder {
        /// Description of failure
        error: String,
    },

    /// Reference data required by a configured rule is missing (fail-closed).
    MissingReferenceData {
        /// Instrument concerned, when the rule is instrument-scoped
        instrument: Option<InstrumentId>,
        /// Which reference datum is missing
        field: ReferenceField,
    },

    /// The instrument's price band is older than the configured maximum age.
    StaleReferenceData {
        /// Instrument whose reference data is stale
        instrument: InstrumentId,
        /// Timestamp of the band, `None` if it carries no timestamp
        as_of: Option<DateTime<Utc>>,
        /// Configured maximum age in milliseconds
        max_age_ms: u64,
    },
}

/// Reference datum whose absence causes a fail-closed refusal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceField {
    /// Scrip price band / circuit limits.
    PriceBand,
    /// Reference (previous close) price, absent or zero.
    ReferencePrice,
    /// Exchange single-order freeze limit for a derivatives instrument.
    FreezeLimit,
    /// Total portfolio equity (non-positive or unavailable).
    TotalEquity,
}

impl fmt::Display for RiskRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GlobalHalted { reason } => {
                write!(f, "Platform trading globally halted: {reason}")
            }
            Self::AlgoHalted { algo_id, reason } => {
                write!(f, "Algorithm '{algo_id}' is halted: {reason}")
            }
            Self::AlgoNotRegistered { algo_id, reason } => {
                write!(f, "Algo ID '{algo_id}' not registered: {reason}")
            }
            Self::MarketCircuitHalt {
                index_symbol,
                stage_pct,
                resume_at,
            } => match resume_at {
                Some(res) => write!(
                    f,
                    "Market-wide circuit halt on {index_symbol} ({stage_pct}%); resumes at {res}"
                ),
                None => write!(
                    f,
                    "Market-wide circuit halt on {index_symbol} ({stage_pct}%); market closed for remainder of day"
                ),
            },
            Self::OtrBreached {
                algo_id,
                current_otr,
                max_otr,
            } => write!(
                f,
                "SEBI OTR threshold breached for algo '{algo_id}': current {current_otr} > limit {max_otr}"
            ),
            Self::ScripOtrBreached {
                instrument,
                current_otr,
                max_otr,
            } => write!(
                f,
                "SEBI scrip OTR threshold breached for {instrument}: current {current_otr} > limit {max_otr}"
            ),
            Self::PriceBandViolation { price, band } => write!(
                f,
                "Price band violation: order price {} outside circuit bounds [{}, {}]",
                price,
                band.lower(),
                band.upper()
            ),
            Self::PriceCollarViolation {
                instrument,
                implied_price,
                reference_price,
                collar_bps,
            } => write!(
                f,
                "Market order price collar violation for {instrument}: worst-case fill {} deviates more than {collar_bps} bps from reference {}",
                implied_price, reference_price
            ),
            Self::MaxNotionalExceeded { requested, limit } => write!(
                f,
                "Order notional {} INR exceeds ceiling {} INR",
                requested, limit
            ),
            Self::MaxQuantityExceeded { requested, limit } => {
                write!(f, "Order quantity {} exceeds ceiling {}", requested, limit)
            }
            Self::FreezeLimitExceeded {
                instrument,
                requested,
                limit,
            } => write!(
                f,
                "Exchange freeze limit exceeded for {instrument}: requested {} > limit {}",
                requested, limit
            ),
            Self::OrderRateExceeded {
                count,
                limit,
                window_ms,
            } => write!(
                f,
                "Rate limit exceeded: {count} orders in {window_ms}ms (max {limit})"
            ),
            Self::DailyLossExceeded {
                current_loss,
                limit,
            } => write!(
                f,
                "Daily loss limit reached: -{} INR exceeds max loss -{} INR",
                current_loss, limit
            ),
            Self::ReduceOnlyViolation { details } => {
                write!(f, "Reduce-only mode violation: {details}")
            }
            Self::PositionConcentrationExceeded {
                instrument,
                position_value,
                total_equity,
                max_pct,
            } => write!(
                f,
                "Position concentration exceeded for {instrument}: {} INR / {} INR equity = {:.1}% > max {:.1}%",
                position_value,
                total_equity,
                (position_value / total_equity) * Decimal::from(100),
                max_pct
            ),
            Self::MisLeverageExceeded {
                gross_exposure,
                total_equity,
                max_leverage,
            } => write!(
                f,
                "MIS leverage exceeded: {} INR exposure / {} INR equity = {:.1}x > max {:.1}x",
                gross_exposure,
                total_equity,
                gross_exposure / total_equity,
                max_leverage
            ),
            Self::MissingStopProtection { reason } => {
                write!(f, "Missing or invalid stop protection: {reason}")
            }
            Self::StopTooTight {
                stop_dist,
                min_sl_dist,
            } => write!(
                f,
                "Initial stop too tight: distance {stop_dist} below minimum {min_sl_dist}"
            ),
            Self::StopTooWide {
                stop_dist,
                max_sl_dist,
            } => write!(
                f,
                "Initial stop too wide: distance {stop_dist} above maximum {max_sl_dist}"
            ),
            Self::StopOutsidePriceCollar {
                stop,
                reference,
                collar_bps,
            } => write!(
                f,
                "Initial stop {stop} sits outside the {collar_bps} bps price collar around reference {reference}"
            ),
            Self::MaxLossExceeded {
                risk_at_stake,
                max_loss,
            } => write!(
                f,
                "Planned max loss {} {} exceeds configured max loss {} {}",
                risk_at_stake.amount(),
                risk_at_stake.currency(),
                max_loss.amount(),
                max_loss.currency()
            ),
            Self::StopWideningDenied {
                current_stop,
                new_stop,
            } => write!(
                f,
                "Trailing stop may not widen: new stop {new_stop} loosens current stop {current_stop}"
            ),
            Self::InvalidOrder { error } => write!(f, "Invalid order: {error}"),
            Self::MissingReferenceData { instrument, field } => match instrument {
                Some(i) => write!(f, "Missing reference data for {i}: {field:?}"),
                None => write!(f, "Missing reference data: {field:?}"),
            },
            Self::StaleReferenceData {
                instrument,
                as_of,
                max_age_ms,
            } => match as_of {
                Some(t) => write!(
                    f,
                    "Stale reference data for {instrument}: as of {t}, max age {max_age_ms} ms"
                ),
                None => write!(
                    f,
                    "Stale reference data for {instrument}: untimestamped, max age {max_age_ms} ms"
                ),
            },
        }
    }
}

impl std::error::Error for RiskRefusal {}

impl RiskRefusal {
    /// Return the regulatory/risk rule name identifying this refusal.
    pub fn rule(&self) -> &'static str {
        match self {
            Self::GlobalHalted { .. } => "global_halt",
            Self::AlgoHalted { .. } => "algo_halt",
            Self::AlgoNotRegistered { .. } => "algo_not_registered",
            Self::MarketCircuitHalt { .. } => "market_circuit_halt",
            Self::OtrBreached { .. } => "sebi_otr_breach",
            Self::ScripOtrBreached { .. } => "sebi_scrip_otr_breach",
            Self::PriceBandViolation { .. } => "price_band_violation",
            Self::PriceCollarViolation { .. } => "price_collar_violation",
            Self::MaxNotionalExceeded { .. } => "max_notional_exceeded",
            Self::MaxQuantityExceeded { .. } => "max_quantity_exceeded",
            Self::FreezeLimitExceeded { .. } => "freeze_limit_exceeded",
            Self::OrderRateExceeded { .. } => "order_rate_exceeded",
            Self::DailyLossExceeded { .. } => "daily_loss_exceeded",
            Self::ReduceOnlyViolation { .. } => "reduce_only_violation",
            Self::PositionConcentrationExceeded { .. } => "position_concentration_exceeded",
            Self::MisLeverageExceeded { .. } => "mis_leverage_exceeded",
            Self::MissingStopProtection { .. } => "missing_stop_protection",
            Self::StopTooTight { .. } => "stop_too_tight",
            Self::StopTooWide { .. } => "stop_too_wide",
            Self::StopOutsidePriceCollar { .. } => "stop_outside_price_collar",
            Self::MaxLossExceeded { .. } => "max_loss_exceeded",
            Self::StopWideningDenied { .. } => "stop_widening_denied",
            Self::InvalidOrder { .. } => "invalid_order",
            Self::MissingReferenceData { .. } => "missing_reference_data",
            Self::StaleReferenceData { .. } => "stale_reference_data",
        }
    }
}
