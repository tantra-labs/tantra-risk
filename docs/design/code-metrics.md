# `vyapaar-risk` — Metrics Reference

Documents `vyapaar-risk/src/metrics.rs`: atomic Prometheus histogram and counters
for observability of risk gate evaluation.

## `RiskMetrics`

Lock-free atomic metrics tracking:

| Metric | Type | Purpose |
|--------|------|---------|
| `risk_eval_duration` | Histogram | Latency distribution of `check_order` calls (`RISK_EVAL_BUCKETS`) |
| `risk_refusals` | Counter | Total refusals by rule name |
| `risk_approvals` | Counter | Total approved orders |
| `algo_halted` | Counter | Per-algo halt events |
| `kill_switch_activated` | Counter | Global kill-switch activations |

## `RISK_EVAL_BUCKETS`

Prometheus histogram buckets configured for sub-15µs SLO monitoring, capturing
latency from nanoseconds to milliseconds.

## Zero-I/O Purity

Metrics are atomic counters/histograms — no disk I/O, no network calls in the
evaluation path. Prometheus scraping happens out-of-band.

## Related

- `vyapaar-risk/src/engine.rs` — `RiskEngine` records metrics on every check.
- `vyapaar-exec/src/executor.rs` — `RiskGatedExecutor` feeds refusal/approval counters.
