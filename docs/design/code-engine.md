# `tantra-risk` — Risk Engine Reference

Documents `tantra-risk/src/engine.rs`: the synchronous, deterministic pre-trade risk
gate that operates between Strategy Decision / Execution Request and the Broker
Executor. Cannot be bypassed by LLMs or strategies.

## `RiskEngine`

```rust
#[derive(Debug, Clone)]
pub struct RiskEngine {
    pub global_state: GlobalState,                          // Active | ReduceOnly | Halted
    pub market_circuit: Option<MarketCircuitHalt>,          // market-wide index circuit breaker
    pub algo_states: HashMap<String, AlgoState>,            // per-algo operational state
    pub algo_trackers: HashMap<String, OtrTracker>,         // per-algo OTR
    pub scrip_trackers: HashMap<InstrumentId, OtrTracker>,  // per-scrip OTR
    pub algo_registry: AlgoIdRegistry,                      // SEBI algo ID registry
    pub refusal_counts: HashMap<String, u64>,               // refusal counters by rule name
    pub cumulative_turnover: HashMap<String, Decimal>,      // daily notional (INR) per algo
    pub cumulative_pnl: HashMap<String, Decimal>,           // daily realized PnL (INR) per algo
    pub otr_config: OtrConfig,
    pub limits: RiskLimits,
    pub price_bands: HashMap<InstrumentId, PriceBand>,      // scrip-wise price bands
    pub order_history: VecDeque<DateTime<Utc>>,             // approved order timestamps (rate limit)
    pub daily_loss: Decimal,                                // cumulative daily realized loss (INR)
}
```

Construction: `RiskEngine::new(limits, otr_config)` and `RiskEngine::default()` (both start
with `GlobalState::Active`, empty trackers, and `RiskLimits::default()` / `OtrConfig::default()`).

## SLOs

| Metric | Target |
|--------|--------|
| `p99.99` latency | < 15 µs |
| `p50` latency | < 1 µs |
| `p99` latency | < 5 µs |
| Throughput | ≥ 50,000 checks/second burst |
| Zero-allocation hot path | `check_order_fast` — no heap allocations |

## Key Methods

### Control & state

| Method | Purpose |
|--------|---------|
| `halt_global(reason: impl Into<String>, timestamp: DateTime<Utc>)` | Trigger platform-wide emergency kill-switch (`GlobalState::Halted`). |
| `resume_global()` | Resume from global halt (`GlobalState::Active`). |
| `set_reduce_only()` | Set `GlobalState::ReduceOnly` (⚠ not enforced — see gap 0002). |
| `trigger_market_circuit(index_symbol, stage_pct, resume_at: Option<DateTime<Utc>>, halted_at)` | Trip market-wide index circuit (10/15/20%). |
| `clear_market_circuit()` | Clear market-wide circuit breaker. |
| `halt_algo(algo_id: &str, reason: impl Into<String>, timestamp: DateTime<Utc>)` | Halt one algo ID (`AlgoState::Halted`). |
| `resume_algo(algo_id: &str)` | Resume one algo ID (`AlgoState::Active`). |
| `set_price_band(instrument: InstrumentId, band: PriceBand)` | Update/register a scrip price band. |

### SEBI algo registry

| Method | Purpose |
|--------|---------|
| `register_algo(algo_id: impl Into<String>)` | Register an algo ID as active. |
| `suspend_algo(algo_id: &str, reason: impl Into<String>)` | Suspend a registered algo ID. |
| `reactivate_algo(algo_id: &str)` | Reactivate a suspended algo ID. |
| `unregister_algo(algo_id: &str)` | Remove an algo ID from the registry. |
| `is_algo_registered(&self, algo_id: &str) -> bool` | True if registered *and* active. |

### Order / decision evaluation

| Method | Purpose |
|--------|---------|
| `check_order(&mut self, order: &OrderRequest, now: DateTime<Utc>) -> Result<(), RiskRefusal>` | Full synchronous pre-trade evaluation (structural validation + `evaluate_core`). |
| `check_order_fast(&mut self, algo_id, instrument, quantity: Quantity, limit_price: Option<Price>, now) -> Result<(), RiskRefusal>` | Zero-allocation hot path (see pipeline divergences below). |
| `check_decision(&mut self, decision: &Decision, now) -> Result<(), RiskRefusal>` | Synchronously check an AI/LLM `Decision` (closed grammar). |
| `validate_price_band(&self, instrument: &InstrumentId, price: Price) -> Result<(), RiskRefusal>` | Standalone band check (approves when no band is set). |
| `check_position_concentration(&self, instrument, position_value: Decimal, total_equity: Decimal) -> Result<(), RiskRefusal>` | Single-scrip concentration vs. equity (`limits.max_position_concentration_pct`). |
| `check_mis_leverage(&self, gross_exposure: Decimal, total_equity: Decimal) -> Result<(), RiskRefusal>` | MIS gross leverage vs. equity (`limits.max_mis_leverage`). |
| `suggest_freeze_slices(&self, instrument, quantity: Quantity) -> u32` | Ceiling-division slice count to fit an order under the freeze limit (1 if already within / unset). |
| `validate_sizing_request(&self, request: &PositionSizingRequest, budget: &RiskBudget) -> Result<(), RiskError>` | Sizing budget gate (rejects on global halt, backfills freeze qty, delegates to `sizing_gate`). |

### Recording & queries

| Method | Purpose |
|--------|---------|
| `record_fill(&mut self, fill: &FillReport)` | Update per-algo + per-scrip OTR trackers and cumulative turnover from the fill. |
| `record_cancellation(&mut self, algo_id: &str, ts: DateTime<Utc>)` | Record mod/cancel on the per-algo tracker. |
| `record_cancellation_for_scrip(&mut self, algo_id, instrument: &InstrumentId, ts)` | Mod/cancel on both per-algo and per-scrip trackers. |
| `record_realized_loss(&mut self, loss_inr: Decimal)` | Add a positive realized loss to `daily_loss`. |
| `record_pnl(&mut self, algo_id: &str, pnl: Decimal)` | Accumulate per-algo PnL; a negative PnL adds to `daily_loss`. |
| `record_refusal(&mut self, refusal: &RiskRefusal)` | Bump `refusal_counts[rule]`. |
| `otr_for_algo(&self, algo_id) -> Option<Decimal>` | Current cumulative OTR for an algo. |
| `otr_for_scrip(&self, instrument: &InstrumentId) -> Option<Decimal>` | Current cumulative OTR for a scrip. |
| `total_refusals(&self) -> u64` / `refusals_for_rule(&self, rule) -> u64` | Refusal statistics. |
| `turnover_for_algo(&self, algo_id) -> Decimal` / `pnl_for_algo(&self, algo_id) -> Decimal` | Per-algo daily turnover / PnL. |

## `evaluate_core_checks` — the full 8-step pipeline

Called by `check_order` / `check_decision` (via `evaluate_core`, which additionally emits an
OpenTelemetry `risk_eval_core` span propagating the W3C `TraceContext` and records metrics):

1. Global kill-switch (`GlobalHalted`)
1.5 Market-wide index circuit breaker (`MarketCircuitHalt`)
2. Per-algo halt with auto-resume cooldown (`halt_cooldown_ms`; `AlgoHalted`)
2.5 SEBI algo registry (`AlgoNotRegistered`)
3. Per-algo 1-min OTR + warning zone (`OtrBreached`)
3.5 Per-scrip OTR (`ScripOtrBreached`)
4. Price band (`PriceBandViolation`)
5. Max quantity (`MaxQuantityExceeded`)
5.5 Exchange freeze limit (`FreezeLimitExceeded`)
6. Max notional (`MaxNotionalExceeded`)
7. Max daily loss — trips the **global** kill-switch on breach (`DailyLossExceeded`)
8. Order rate limit — event-time sliding window (`OrderRateExceeded`)

## `check_order_fast` — documented divergences

`check_order_fast_checks` enforces the same rules with `HashMap::get()` (no allocation on
the entry path) but intentionally differs from `evaluate_core_checks`:

- **Per-scrip OTR is not checked on the hot path** — the scrip placement is recorded only
  after all other checks pass (`scrip_tracker.record_order_placement(now)` at the end) and
  is never compared against `max_otr`.
- **OTR warning zone is skipped** (no `warning_ratio` logging).
- **Halt cooldown auto-resume is skipped** — `AlgoState::Halted` is rejected unconditionally.

Because of this, the two paths can disagree on scrip OTR enforcement; tracked in
[`docs/gaps/0003-check-order-fast-otr-divergence.md`](../gaps/0003-check-order-fast-otr-divergence.md).

## `check_decision` — decision grammar mapping

`Decision::Hold` → approve; `PlaceOrder` → full pipeline on `Exchange::Nse`; `CancelOrder` /
`CancelAll` / `ModifyOrder` → record the order operation and approve. A structurally invalid
`Decision` is rejected with `RiskRefusal::InvalidOrder`.

## Zero-I/O & Zero-Floats

- 100% synchronous, in-memory — no async, no network, no disk.
- All timestamps injected explicitly via `DateTime<Utc>`; no `Utc::now()` / `SystemTime::now()`.
  `Instant::now()` is used only to measure evaluation latency for metrics, never for state.
- All financial values use `rust_decimal::Decimal` (`Price` / `Quantity` wrappers from `tantra-core`).

## Related

- `tantra-risk/src/otr.rs` — `OtrTracker`, sliding-window OTR
- `tantra-risk/src/limits.rs` — `RiskLimits`, `OrderRateLimit`
- `tantra-risk/src/algo_registry.rs` — `AlgoIdRegistry`
- `tantra-risk/src/state.rs` — `GlobalState`, `AlgoState`, `MarketCircuitHalt`
- `tantra-risk/src/refusal.rs` — `RiskRefusal`
- `tantra-risk/src/sizing_gate.rs` — `validate_sizing_request`
