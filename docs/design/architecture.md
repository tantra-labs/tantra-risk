# `vyapaar-risk` — DDD Architecture & Pre-Trade Regulatory Gate

## 1. Bounded Context & Responsibility

`vyapaar-risk` (Tier 1A, ported from `honba-mon`) is the **authoritative, synchronous
pre-trade regulatory risk gate** for Indian equities and derivatives (NSE / BSE / NFO).
Every order placement, modification, cancellation, AI `Decision`, and position-sizing
request must pass through `RiskEngine` before reaching execution. It is pure and
synchronous — it cannot be bypassed by LLMs or strategies.

## 2. Performance & Purity SLOs

| Metric | Target |
|--------|--------|
| `p99.99` latency | < 15 µs (under burst load) |
| `p50` latency | < 1 µs |
| `p99` latency | < 5 µs |
| Throughput | ≥ 50,000 checks/second burst |
| Allocation invariant | `check_order_fast` — zero heap allocations on the hot path |

- **Zero-I/O**: 100% synchronous, in-memory evaluation — no async, no network, no disk.
- **Temporal determinism**: all state transitions receive an explicit `DateTime<Utc>`;
  no `Utc::now()` / `SystemTime::now()` anywhere in `src/`. (`Instant::now()` appears
  only inside `evaluate_core` / `check_order_fast` to measure evaluation latency for
  metrics — never for state.)
- **Zero-Floats**: all notional, price-band, loss, and OTR calculations use
  `rust_decimal::Decimal`.

## 3. Domain Aggregates & Services (`src/`)

| Module | Responsibility |
|--------|----------------|
| `engine.rs` | `RiskEngine` root aggregate: global/per-algo state, OTR trackers, limits, and all check paths. |
| `algo_registry.rs` | `AlgoIdRegistry` — SEBI Retail Algo Circular registry (`Active \| Suspended { reason } \| NotRegistered`). |
| `state.rs` | `GlobalState` (`Active \| ReduceOnly \| Halted`), `AlgoState` (`Active \| Halted { reason, halted_at }`), `MarketCircuitHalt`. |
| `otr.rs` | `OtrTracker` / `OtrConfig` — sliding-window Order-to-Trade Ratio (1-min / 5-min windows, cumulative ratio, warning zone). |
| `limits.rs` | `RiskLimits`, `OrderRateLimit` — notional, quantity, daily-loss, freeze, concentration, MIS-leverage, rate limits. |
| `refusal.rs` | `RiskRefusal` — typed, `serde`-serializable refusal reasons with a stable `rule()` name per variant. |
| `sizing_gate.rs` | `validate_sizing_request` — dynamic position-sizing risk budget gate; `SizingRiskRefusal`. |
| `metrics.rs` | `RiskMetrics` — Prometheus histogram/counters/gauge; `GLOBAL_RISK_METRICS` singleton. |
| `tool_hints.rs` | `ToolRiskHint` / `risk_classification` — MCP tool-annotation hooks (`readOnlyHint` / `destructiveHint` / `idempotentHint`). |
| `error.rs` | `RiskError` — `InvalidConfiguration` and `SizingRefusal(#[from] SizingRiskRefusal)`. |

## 4. Evaluation Pipeline

### 4.1 `evaluate_core_checks` — full path (via `check_order` / `check_decision`)

`RiskEngine::check_order(&OrderRequest, now)` and `check_decision(&Decision, now)` both
route through `evaluate_core` → `evaluate_core_checks`. Checks execute in source order:

1. **Global kill-switch** — `GlobalState::Halted` → `RiskRefusal::GlobalHalted`.
1.5 **Market-wide index circuit breaker** — `market_circuit` with `resume_at`:
    rejects while `now < resume_at` or when `resume_at` is `None` (day-closing circuit)
    → `RiskRefusal::MarketCircuitHalt`.
2. **Per-algo halt** — `AlgoState::Halted`: if `otr_config.halt_cooldown_ms > 0` and the
    cooldown has expired, the algo is auto-resumed; otherwise → `RiskRefusal::AlgoHalted`.
2.5 **SEBI algo registry** — `algo_registry.is_registered_and_active(algo_id)` must pass
    → `RiskRefusal::AlgoNotRegistered`.
3. **Per-algo 1-min OTR** — records the placement, computes `window_otr_1min(now)`, and
    once `total_order_operations() >= min_orders_before_enforcement`, a ratio above
    `max_otr` trips the per-algo circuit breaker (`halt_algo`) → `RiskRefusal::OtrBreached`.
    A `warning_ratio()` above `warning_threshold` (but ≤ `max_otr`) is logged as a warning.
3.5 **Per-scrip OTR** — same enforcement on the scrip tracker (keyed by `InstrumentId`)
    → `RiskRefusal::ScripOtrBreached`.
4. **Price band** — limit price must fall within `price_bands[instrument]` when set
    → `RiskRefusal::PriceBandViolation`.
5. **Max quantity** — `limits.max_quantity_per_order` → `RiskRefusal::MaxQuantityExceeded`.
5.5 **Exchange freeze limit** — `limits.freeze_limits[instrument]` → `RiskRefusal::FreezeLimitExceeded`.
6. **Max notional** — `limits.max_notional_per_order`, evaluated at the limit price (or the
    band reference price for market orders) → `RiskRefusal::MaxNotionalExceeded`.
7. **Max daily loss** — `limits.max_daily_loss`; on breach trips the **global** kill-switch
    → `RiskRefusal::DailyLossExceeded`.
8. **Order rate limit** — event-time sliding window over `order_history`
    (`OrderRateLimit { max_orders, window_ms }`) → `RiskRefusal::OrderRateExceeded`.

On success the timestamp is appended to `order_history`. Every refusal is recorded via
`record_refusal` and exported to the Prometheus `risk_breaches_total` / `risk_orders_evaluated_total`.

### 4.2 `check_order_fast` — zero-allocation hot path

`RiskEngine::check_order_fast(algo_id, instrument, quantity, limit_price, now)` executes
the same rule set through `check_order_fast_checks` with `HashMap::get()` instead of
`HashMap::entry()` to avoid allocations. It **diverges** from the full path by:

- **skipping the per-scrip OTR circuit-breaker check** — the scrip placement is recorded
  only *after* all other checks pass, and never compared against `max_otr`;
- **skipping the OTR warning zone** (`warning_ratio`);
- **skipping the halt cooldown auto-resume** — a halted algo is rejected unconditionally.

These divergences are tracked in
[`docs/gaps/0003-check-order-fast-otr-divergence.md`](../gaps/0003-check-order-fast-otr-divergence.md).

### 4.3 `check_decision` — closed `Decision` grammar

`check_decision(&Decision, now)` validates the decision structurally
(`decision.validate()`), then maps the `vyapaar_core::Decision` grammar:

| Decision variant | Action |
|------------------|--------|
| `Hold` | Approve. |
| `PlaceOrder` | Builds `InstrumentId::new(Exchange::Nse, symbol)` and runs the full `evaluate_core` pipeline. |
| `CancelOrder` / `CancelAll` | Records the cancellation on the OTR tracker (when `algo_id` present); approve. |
| `ModifyOrder` | Records the order operation; approve. |

### 4.4 Sizing gate

`RiskEngine::validate_sizing_request(&PositionSizingRequest, &RiskBudget)` first rejects
on `GlobalState::Halted` (`SizingRiskRefusal::PlatformHalted`), backfills
`freeze_limit_qty` from `limits.freeze_limits` when the budget omits it, then delegates
to `sizing_gate::validate_sizing_request` (entry-price/equity sanity, stop-loss/side
consistency, portfolio risk cap, lot-size multiple, freeze/max-qty/max-notional caps).

## 5. SLO monitoring

`RiskMetrics` records evaluation latency into `risk_eval_duration_seconds`
(`RISK_EVAL_BUCKETS` = 100ns/250ns/500ns/1µs/5µs/15µs/50µs/100µs), approved/rejected
decisions into `risk_orders_evaluated_total{algo_id,result}`, rule breaches into
`risk_breaches_total{algo_id,rule_name}`, and the live OTR into
`risk_otr_current_ratio{algo_id,scrip}`.

## 6. Open gaps

See [`docs/gaps/README.md`](../gaps/README.md) — four source-verified gaps are filed
(unenforced `price_collar_bps`, unenforced reduce-only mode, `check_order_fast` OTR
divergence, dead OTR window config knobs).
