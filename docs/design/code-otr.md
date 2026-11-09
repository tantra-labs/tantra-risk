# `vyapaar-risk` — OTR Tracker Reference

Documents `vyapaar-risk/src/otr.rs`: SEBI Order-to-Trade Ratio tracker with
sliding-window computation and per-algo circuit breaking.

## `OtrConfig`

| Field | Default | Purpose |
|-------|---------|---------|
| `max_otr` | 50 | Maximum allowed Order-to-Trade Ratio |
| `min_orders_before_enforcement` | 20 | Grace period before hard clamping |
| `warning_threshold` | 40 | Soft warning at 80% of max |
| `window_1min_ms` | 60_000 | 1-minute sliding window |
| `window_5min_ms` | 300_000 | 5-minute sliding window |
| `halt_cooldown_ms` | 0 | Cooling-off after breach (0 = no auto-resume) |

## `OrderEvent` / `OrderEventKind`

| Kind | Meaning |
|------|---------|
| `Placement` | New order placed |
| `ModOrCancel` | Order modified or cancelled |
| `Fill` | Trade executed / fill received |

## `OtrTracker`

Tracks per-algo OTR state using a circular ring buffer of event timestamps over
1-minute, 5-minute, and session windows. Supports both cumulative totals and
sliding-window OTR computation.

## Circuit Breaking

When OTR exceeds `max_otr` after `min_orders_before_enforcement`:
- Algo is halted (state → `Halted`)
- `halt_cooldown_ms` governs auto-resume (0 = manual resume only)
- Warning emitted at `warning_threshold` before hard breach

## Zero-Floats Policy

All OTR ratios and thresholds use `rust_decimal::Decimal` — exact decimal
arithmetic, no floating point.

## Related

- `vyapaar-risk/src/engine.rs` — `RiskEngine` uses `OtrTracker` for per-algo and per-scrip tracking.
- `vyapaar-exec/src/router/src/otr.rs` — `OtrLedger` for SOR OTR balancing.
