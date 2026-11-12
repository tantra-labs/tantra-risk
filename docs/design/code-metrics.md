# `vyapaar-risk` — Metrics Reference

Documents `vyapaar-risk/src/metrics.rs`: Prometheus telemetry for pre-trade risk gate
observability. Metrics are collected with the **`prometheus` crate (0.13)** — not
hand-rolled atomic/lock-free structures. The collectors themselves are internally
atomic; no locks or extra allocations are introduced by the crate on the record path.

## `RiskMetrics`

```rust
#[derive(Clone)]
pub struct RiskMetrics {
    registry: Registry,
    pub eval_duration: Histogram,
    pub orders_evaluated: CounterVec,   // labels: ["algo_id", "result"]
    pub breaches: CounterVec,           // labels: ["algo_id", "rule_name"]
    pub otr_ratio: GaugeVec,            // labels: ["algo_id", "scrip"]
}
```

| Metric | Type | Labels | Purpose |
|--------|------|--------|---------|
| `risk_eval_duration_seconds` | `Histogram` | — | Evaluation latency, sub-microsecond buckets |
| `risk_orders_evaluated_total` | `CounterVec` | `algo_id`, `result` (`"approved"` / `"rejected"`) | Evaluated decisions |
| `risk_breaches_total` | `CounterVec` | `algo_id`, `rule_name` | Individual regulatory/risk rule violations |
| `risk_otr_current_ratio` | `GaugeVec` | `algo_id`, `scrip` | Active rolling Order-to-Trade Ratio |

> **Note**: counters named `risk_refusals`, `risk_approvals`, `algo_halted`, and
> `kill_switch_activated` do **not** exist. Refusal/approval/breach observability is
> carried by `risk_orders_evaluated_total` and `risk_breaches_total`; per-rule refusal
> counts are also tracked in-process on `RiskEngine::refusal_counts`
> (`total_refusals()` / `refusals_for_rule()`).

## `RISK_EVAL_BUCKETS`

```rust
pub const RISK_EVAL_BUCKETS: &[f64] = &[
    0.000_000_100, // 100 ns
    0.000_000_250, // 250 ns
    0.000_000_500, // 500 ns
    0.000_001_000, // 1 µs
    0.000_005_000, // 5 µs
    0.000_015_000, // 15 µs (Pre-Trade SLO ceiling)
    0.000_050_000, // 50 µs
    0.000_100_000, // 100 µs
];
```

Configured specifically to monitor the sub-15 µs SLO (`p99.99 < 15 µs`, `p50 < 1 µs`,
`p99 < 5 µs`, ≥ 50k checks/s burst).

## API

| Method | Signature | Purpose |
|--------|-----------|---------|
| `new` | `fn new() -> Result<Self, prometheus::Error>` | Build + register all four metrics. |
| `record_evaluation_duration` | `fn record_evaluation_duration(&self, duration_seconds: f64)` | Observe into `risk_eval_duration_seconds`. |
| `record_order_evaluated` | `fn record_order_evaluated(&self, algo_id: &str, approved: bool)` | Increment `risk_orders_evaluated_total{result}`. |
| `record_breach` | `fn record_breach(&self, algo_id: &str, rule_name: &str)` | Increment `risk_breaches_total`. |
| `set_otr_ratio` | `fn set_otr_ratio(&self, algo_id: &str, scrip: &str, ratio: f64)` | Set `risk_otr_current_ratio` gauge. |
| `registry` | `fn registry(&self) -> &Registry` | Access the Prometheus registry. |
| `export_metrics` | `fn export_metrics(&self) -> Result<String, prometheus::Error>` | Gather + encode as Prometheus text format. |

## Global singleton

```rust
pub static GLOBAL_RISK_METRICS: LazyLock<RiskMetrics> = LazyLock::new(|| {
    RiskMetrics::new().expect("Failed to initialize global pre-trade risk metrics collector")
});

#[inline]
pub fn global_metrics() -> &'static RiskMetrics { &GLOBAL_RISK_METRICS }
```

`engine.rs` records into this singleton on every evaluation: `evaluate_core` and
`check_order_fast` observe latency, `record_order_evaluated(algo_id, approved)` on success,
and `record_breach(algo_id, refusal.rule())` on refusal. The OTR gauge is set with
`scrip = "*"` for the per-algo ratio and the instrument symbol for the per-scrip ratio.

## Zero-I/O Purity

No disk I/O or network calls in the evaluation path — Prometheus scraping happens
out-of-band via `registry()` / `export_metrics()`. Note that the latency buckets are
float-based (`f64` seconds), which is metric telemetry only — **all financial state** in the
crate uses `rust_decimal::Decimal`.

## Related

- `vyapaar-risk/src/engine.rs` — `RiskEngine` records metrics on every check.
- `vyapaar-risk/src/refusal.rs` — `RiskRefusal::rule()` supplies `rule_name` labels.
