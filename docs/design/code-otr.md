# `vyapaar-risk` — OTR Tracker Reference

Documents `vyapaar-risk/src/otr.rs`: the SEBI Order-to-Trade Ratio tracker with
sliding-window computation and per-algo / per-scrip circuit breaking. Under the SEBI
regulatory framework (and the Retail Algo Circular effective August 1, 2025), algorithmic
orders must maintain acceptable OTR levels.

## `OtrConfig`

| Field | Default | Purpose |
|-------|---------|---------|
| `max_otr` | 50 | Maximum allowed Order-to-Trade Ratio |
| `min_orders_before_enforcement` | 20 | Order-operation grace period before hard clamping |
| `warning_threshold` | 40 | Soft warning at 80% of max (warn only, no halt) |
| `window_1min_ms` | 60_000 | Declared 1-minute window (⚠ never read — see gap 0004) |
| `window_5min_ms` | 300_000 | Declared 5-minute window (⚠ never read — see gap 0004) |
| `halt_cooldown_ms` | 0 | Cooling-off after a breach halt before auto-resume (0 = no auto-resume) |

## `OrderEvent` / `OrderEventKind`

`OrderEvent { ts: DateTime<Utc>, kind: OrderEventKind }` — a timestamped order event.

| Kind | Meaning |
|------|---------|
| `Placement` | New order placed |
| `ModOrCancel` | Order modified or cancelled |
| `Fill` | Trade executed / fill received |

## `OtrTracker`

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
pub struct OtrTracker {
    pub orders_placed: u64,
    pub orders_modified: u64,
    pub orders_cancelled: u64,
    pub trades_executed: u64,
    #[serde(skip)]
    events: VecDeque<OrderEvent>,   // ring buffer (growable) of recent events
}
```

The event store is a **`std::collections::VecDeque<OrderEvent>` ring buffer** (not a
fixed-size circular buffer). Stale events are evicted from the front by
`window_otr(now, window_ms)`.

| Method | Signature | Purpose |
|--------|-----------|---------|
| `new` | `fn new() -> Self` | Empty tracker. |
| `record_order_placement` | `fn record_order_placement(&mut self, ts: DateTime<Utc>)` | Push a `Placement` event. |
| `record_order_mod_or_cancel` | `fn record_order_mod_or_cancel(&mut self, ts)` | Push a `ModOrCancel` event. |
| `record_fill` | `fn record_fill(&mut self, ts: DateTime<Utc>)` | Push a `Fill` event. |
| `total_order_operations` | `fn total_order_operations(&self) -> u64` | `placements + modifications + cancellations`. |
| `current_otr` | `fn current_otr(&self) -> Decimal` | **Cumulative** ratio: `ops / trades` (trades floor 1). |
| `window_otr` | `fn window_otr(&mut self, now: DateTime<Utc>, window_ms: u64) -> Decimal` | Evicts expired events, then sliding-window ratio. |
| `window_otr_1min` | `fn window_otr_1min(&mut self, now) -> Decimal` | 1-minute window (`window_otr(now, 60_000)`). |
| `window_otr_5min` | `fn window_otr_5min(&mut self, now) -> Decimal` | 5-minute window (`window_otr(now, 300_000)`). |
| `is_breached` | `fn is_breached(&self, config: &OtrConfig) -> bool` | Cumulative OTR above `max_otr` after the grace period. |
| `warning_ratio` | `fn warning_ratio(&self, config: &OtrConfig) -> Option<Decimal>` | `Some(otr)` when `warning_threshold < otr <= max_otr`. |
| `event_count` | `fn event_count(&self) -> usize` | Number of events currently buffered. |

## Circuit Breaking

In `evaluate_core_checks` (full path), when the 1-minute window OTR exceeds `max_otr` after
`min_orders_before_enforcement`:

- The per-algo circuit breaker is tripped (`RiskEngine::halt_algo` → `AlgoState::Halted`).
- `halt_cooldown_ms` governs automatic resumption (0 = manual resume only).
- A warning is emitted at `warning_threshold` (via `warning_ratio`) before the hard breach.

The same enforcement applies per scrip (keyed by `InstrumentId`). `check_order_fast` skips
the scrip OTR check and the warning zone — see
[`docs/gaps/0003-check-order-fast-otr-divergence.md`](../gaps/0003-check-order-fast-otr-divergence.md).

## Config knob caveat

`OtrConfig::window_1min_ms` and `window_5min_ms` are **never consulted** by `otr.rs` or
`engine.rs`: `window_otr_1min` / `window_otr_5min` hardcode `60_000` / `300_000` ms, and the
engine enforces only the 1-minute window. Setting the config fields has no effect
([`docs/gaps/0004-otr-window-config-dead.md`](../gaps/0004-otr-window-config-dead.md)).

## Zero-Floats Policy

All OTR ratios and thresholds use `rust_decimal::Decimal` — exact decimal arithmetic, no
floating point.

## Zero-I/O / Temporal Determinism

Pure in-memory `VecDeque`; every method receives an explicit `DateTime<Utc>` — no wall-clock
queries.

## Related

- `vyapaar-risk/src/engine.rs` — `RiskEngine` uses `OtrTracker` for per-algo and per-scrip tracking.
- `vyapaar-risk/src/refusal.rs` — `RiskRefusal::OtrBreached`, `RiskRefusal::ScripOtrBreached`.
