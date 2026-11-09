# `vyapaar-risk` — Risk Limits Reference

Documents `vyapaar-risk/src/limits.rs`: configurable risk limits applied
synchronously to all outgoing orders and decisions.

## `OrderRateLimit`

| Field | Type | Purpose |
|-------|------|---------|
| `max_orders` | `u32` | Maximum allowed orders in the window |
| `window_ms` | `u64` | Window duration in milliseconds |

## `RiskLimits`

| Field | Type | Purpose |
|-------|------|---------|
| `max_notional_per_order` | `Option<Decimal>` | Max notional value (INR) per single order |
| `max_quantity_per_order` | `Option<Quantity>` | Max quantity per single order |
| `max_daily_loss` | `Option<Decimal>` | Max daily cumulative loss (INR) |
| `order_rate_limit` | `Option<OrderRateLimit>` | Global order rate limit |
| `price_collar_bps` | `Option<u32>` | Market order deviation collar (basis points) |
| `freeze_limits` | `HashMap<InstrumentId, Quantity>` | Per-scrip exchange freeze quantity ceilings |
| `max_position_concentration_pct` | `Option<Decimal>` | Max single-scrip position concentration |
| `max_mis_leverage` | `Option<Decimal>` | Max gross intraday (MIS) leverage |

### Freeze Limit Examples

| Instrument | Freeze Limit |
|------------|-------------|
| NIFTY | 1,800 |
| BANKNIFTY | 900 |

## Legacy Field Aliases

| Legacy Field (`vyapaar-ledger`) | Canonical Field (`vyapaar-risk`) |
|--------------------------------|----------------------------------|
| `max_notional` (f64) | `max_notional_per_order` (Decimal) |
| `max_participation` (f64) | `max_quantity_per_order` (Quantity) |
| `order_rate` (OrderRateLimit) | `order_rate_limit` (OrderRateLimit) |
| `stale_after_ms` (u64) | tracked via circuit breaker |

## Zero-Floats Policy

All notional, price, quantity, and loss values use `rust_decimal::Decimal`.

## Related

- `vyapaar-risk/src/engine.rs` — `RiskEngine` applies `RiskLimits` on every check.
- `vyapaar-core/src/types.rs` — `Price`, `Quantity`, `PriceBand`.
- `vyapaar-core/src/ids.rs` — `InstrumentId`.
