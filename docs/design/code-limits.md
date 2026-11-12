# `tantra-risk` — Risk Limits Reference

Documents `tantra-risk/src/limits.rs`: configurable hard risk limits applied synchronously
to all outgoing orders and decisions.

## `OrderRateLimit`

| Field | Type | Purpose |
|-------|------|---------|
| `max_orders` | `u32` | Maximum allowed orders in the window |
| `window_ms` | `u64` | Window duration in milliseconds |

## `RiskLimits`

| Field | Type | Purpose | Enforced |
|-------|------|---------|----------|
| `max_notional_per_order` | `Option<Decimal>` | Max notional value (INR) per single order | ✅ engine step 6 |
| `max_quantity_per_order` | `Option<Quantity>` | Max quantity per single order | ✅ engine step 5 |
| `max_daily_loss` | `Option<Decimal>` | Max daily cumulative loss (INR); trips global kill-switch on breach | ✅ engine step 7 |
| `order_rate_limit` | `Option<OrderRateLimit>` | Global event-time order rate limit | ✅ engine step 8 |
| `price_collar_bps` | `Option<u32>` | Allowed market-order deviation collar in basis points (1 bp = 0.01%) | ❌ **declared only** — see gap 0001 |
| `freeze_limits` | `HashMap<InstrumentId, Quantity>` | Per-scrip exchange single-order quantity ceilings | ✅ engine step 5.5 |
| `max_position_concentration_pct` | `Option<Decimal>` | Max single-scrip position concentration (% of equity) | ✅ `check_position_concentration` |
| `max_mis_leverage` | `Option<Decimal>` | Max gross intraday (MIS) leverage vs. equity | ✅ `check_mis_leverage` |

### Freeze Limit Examples

| Instrument | Freeze Limit |
|------------|-------------|
| NIFTY | 1,800 |
| BANKNIFTY | 900 |

## `RiskLimits::is_valid()`

Structural sanity check (`-> bool`): rejects `max_notional_per_order <= 0`, zero/empty
`OrderRateLimit` (`max_orders == 0 || window_ms == 0`), `price_collar_bps == 0`, and
non-positive freeze quantities.

## Legacy Field Aliases (from deprecated `tantra-ledger::RiskLimits`)

| Legacy Field (`tantra-ledger`) | Canonical Field (`tantra-risk`) |
|--------------------------------|----------------------------------|
| `max_notional` (f64) | `max_notional_per_order` (Decimal) |
| `max_participation` (f64) | `max_quantity_per_order` (Quantity) |
| `order_rate` (OrderRateLimit) | `order_rate_limit` (OrderRateLimit) |
| `stale_after_ms` (u64) | tracked via separate circuit breaker |

`max_notional` and `max_quantity` are also accepted as `serde` aliases on the canonical
fields. Note: **`price_collar_bps` is validated in `is_valid()` but never consulted by any
check path** in `engine.rs` — the market-order deviation collar is not applied
([`docs/gaps/0001-price-collar-unenforced.md`](../gaps/0001-price-collar-unenforced.md)).

## Zero-Floats Policy

All notional, price, quantity, and loss values use `rust_decimal::Decimal`.

## Related

- `tantra-risk/src/engine.rs` — `RiskEngine` applies `RiskLimits` on every check.
- `tantra-core/src/types.rs` — `Price`, `Quantity`, `PriceBand`.
- `tantra-core/src/ids.rs` — `InstrumentId`.
