# `vyapaar-risk` — Risk Engine Reference

Documents `vyapaar-risk/src/engine.rs`: the synchronous, deterministic pre-trade
risk gate that operates between Strategy Decision / Execution Request and the
Broker Executor. Cannot be bypassed by LLMs or strategies.

## `RiskEngine`

```rust
pub struct RiskEngine {
    pub global_state: GlobalState,           // Active, ReduceOnly, Halted
    pub market_circuit: Option<MarketCircuitHalt>,
    pub algo_states: HashMap<String, AlgoState>,
    pub algo_trackers: HashMap<String, OtrTracker>,
    pub scrip_trackers: HashMap<InstrumentId, OtrTracker>,
    pub algo_registry: AlgoIdRegistry,
    pub refusal_counts: HashMap<String, u64>,
    pub cumulative_turnover: HashMap<String, Decimal>,
    pub cumulative_pnl: HashMap<String, Decimal>,
    pub otr_config: OtrConfig,
    pub limits: RiskLimits,
    pub price_bands: HashMap<InstrumentId, PriceBand>,
    pub order_history: VecDeque<DateTime<Utc>>,
    pub daily_loss: Decimal,
}
```

## SLOs

| Metric | Target |
|--------|--------|
| `p99.99` latency | < 15 µs |
| `p50` latency | < 1 µs |
| `p99` latency | < 5 µs |
| Throughput | ≥ 50,000 checks/second burst |
| Zero-allocation hot path | `check_order_fast` — no heap allocations |

## Key Methods

| Method | Purpose |
|--------|---------|
| `check_order(&mut self, order, now)` | Full synchronous pre-trade evaluation |
| `check_order_fast(...)` | Zero-allocation fast path |
| `register_algo(algo_id)` | Register a SEBI Algo ID |
| `halt_algo(algo_id, reason)` | Per-algo emergency halt |
| `global_kill_switch(active)` | Platform-wide emergency suspension |
| `record_fill(fill, now)` | Update OTR trackers on fill |
| `record_cancel(algo_id, now)` | Track cancellation in OTR |

## Zero-I/O & Zero-Floats

- 100% synchronous, in-memory — no async, no network, no disk
- All timestamps injected explicitly via `DateTime<Utc>`
- All financial values use `rust_decimal::Decimal`

## Related

- `vyapaar-risk/src/otr.rs` — `OtrTracker`, sliding-window OTR
- `vyapaar-risk/src/limits.rs` — `RiskLimits`, `OrderRateLimit`
- `vyapaar-risk/src/algo_registry.rs` — `AlgoIdRegistry`
- `vyapaar-risk/src/sizing_gate.rs` — `validate_sizing_request`
- `vyapaar-exec/src/executor.rs` — `RiskGatedExecutor` wraps any `BrokerExecutor`
