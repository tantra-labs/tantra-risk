# `tantra-risk` — Refusal & State Reference

Documents `tantra-risk/src/refusal.rs` and `tantra-risk/src/state.rs`.

## `RiskRefusal` (`refusal.rs`)

The typed refusal returned when a decision or order fails pre-trade risk checks. Deliberately
returns large aggregate payloads by value so the refusal reason is fully diagnosable on the
hot path; `clippy::result_large_err` is consciously accepted at the crate level
(`#![allow(clippy::result_large_err)]` in `lib.rs`). Every variant is `serde`-serializable
(tagged `reason` / `details`, `snake_case`) and exposes a stable `&'static str` via `rule()`.

| Variant | `rule()` | Fields |
|---------|----------|--------|
| `GlobalHalted` | `global_halt` | `reason: String` |
| `AlgoHalted` | `algo_halt` | `algo_id: String`, `reason: String` |
| `AlgoNotRegistered` | `algo_not_registered` | `algo_id: String`, `reason: String` |
| `MarketCircuitHalt` | `market_circuit_halt` | `index_symbol: String`, `stage_pct: u8`, `resume_at: Option<DateTime<Utc>>` |
| `ScripOtrBreached` | `sebi_scrip_otr_breach` | `instrument: InstrumentId`, `current_otr: Decimal`, `max_otr: Decimal` |
| `OtrBreached` | `sebi_otr_breach` | `algo_id: String`, `current_otr: Decimal`, `max_otr: Decimal` |
| `PriceBandViolation` | `price_band_violation` | `price: Price`, `band: PriceBand` |
| `PriceCollarViolation` | `price_collar_violation` | `instrument`, `implied_price: Price`, `reference_price: Price`, `collar_bps: u32` |
| `MaxNotionalExceeded` | `max_notional_exceeded` | `requested: Decimal`, `limit: Decimal` |
| `FreezeLimitExceeded` | `freeze_limit_exceeded` | `instrument: InstrumentId`, `requested: Quantity`, `limit: Quantity` |
| `MaxQuantityExceeded` | `max_quantity_exceeded` | `requested: Quantity`, `limit: Quantity` |
| `OrderRateExceeded` | `order_rate_exceeded` | `count: u32`, `limit: u32`, `window_ms: u64` |
| `DailyLossExceeded` | `daily_loss_exceeded` | `current_loss: Decimal`, `limit: Decimal` |
| `ReduceOnlyViolation` | `reduce_only_violation` | `details: String` |
| `PositionConcentrationExceeded` | `position_concentration_exceeded` | `instrument`, `position_value: Decimal`, `total_equity: Decimal`, `max_pct: Decimal` |
| `MisLeverageExceeded` | `mis_leverage_exceeded` | `gross_exposure: Decimal`, `total_equity: Decimal`, `max_leverage: Decimal` |
| `InvalidOrder` | `invalid_order` | `error: String` |

`RiskRefusal` also implements `Display` (a human-readable one-line reason) and
`std::error::Error`.

## `state.rs`

### `GlobalState`

```rust
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, Default)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum GlobalState {
    #[default] Active,
    ReduceOnly, // order-opening BUYs refused on every path; SELL/cancel pass
    Halted { reason: String, halted_at: DateTime<Utc> },
}
```

Helpers: `is_active()` (matches `Active`), `is_halted()` (matches `Halted { .. }`).

### `AlgoState`

```rust
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AlgoState {
    #[default] Active,
    Halted { reason: String, halted_at: DateTime<Utc> },
}
```

There is **no** `ReduceOnly` variant on `AlgoState` — per-algo operational state is strictly
`Active | Halted`. Helpers: `is_active()`, `is_halted()`.

### `MarketCircuitHalt`

```rust
pub struct MarketCircuitHalt {
    pub index_symbol: String,            // e.g. "NIFTY 50", "SENSEX"
    pub stage_pct: u8,                   // 10, 15, or 20
    pub halted_at: DateTime<Utc>,
    pub resume_at: Option<DateTime<Utc>>, // None = market closed for remainder of day
}
```

## State enforcement notes

- `GlobalState::Halted` rejects everything; `GlobalState::ReduceOnly` refuses
  order-opening BUYs on both `evaluate_core_checks` and `check_order_fast_checks`, refuses
  `ModifyOrder` and new position sizing (`validate_sizing_request`), and approves reducing
  SELLs and cancels — all with `RiskRefusal::ReduceOnlyViolation` /
  `SizingRiskRefusal::ReduceOnlyViolation` (plan 0002, gap 0002).
- `AlgoState::Halted` rejects until `halt_cooldown_ms` elapses, then auto-resumes — identical
  on both check paths (plan 0002, gap 0003).
- State transitions are explicit methods on `RiskEngine` (`halt_global` / `resume_global` /
  `set_reduce_only` / `halt_algo` / `resume_algo` / `trigger_market_circuit` /
  `clear_market_circuit`) — see `code-engine.md`.

## Zero-I/O & Zero-Floats

All state types are plain data — no async, no I/O, no floats. Transitions are synchronous
and deterministic, with explicit `DateTime<Utc>` timestamps and no wall-clock queries.

## Related

- `tantra-risk/src/engine.rs` — `RiskEngine` holds `global_state`, `algo_states`, `market_circuit`.
- `tantra-risk/src/algo_registry.rs` — `AlgoIdRegistry` / `AlgoRegistrationStatus` (registration status, distinct from `AlgoState`).
