# 0013 — Unbounded Prometheus label cardinality from caller-supplied `algo_id` / scrip, recorded before validation

**Status**: open
**Filed**: 2026-11-23 — workspace design review (root plan 0021) · **Kind**: design-flaw · **Severity**: medium
**Plan**: [0003](../plans/0003-design-review-2026-11.md)

## What's stubbed or hardcoded

- `src/engine.rs:~438-452`: `check_order` calls `metrics.record_order_evaluated(&order.algo_id, false)` / `record_breach(&order.algo_id, ..)` for empty or arbitrary algo ids before the registry check.
- `src/metrics.rs:~126-132`: `set_otr_ratio(algo_id, scrip, ..)` and `with_label_values(&[algo_id, ..])` create a new time series per distinct string; series live for the process life.
- `src/engine.rs:~606`: the `scrip` label is every instrument symbol that reaches the gate.
- `refusal_counts` is bounded by rule name, but `algo_trackers` is not (see [0008](0008-bounded-rate-windows.md)).

## Why

An AI, a user, or a bug spraying random algo ids creates unlimited series, leaking memory and blowing up the scrape. Metrics are recorded before the caller has been validated, so the cheapest input to forge is also the one that costs the most.

## How to close it

1. Label only registered algo ids; map every other value to the constant label `"unregistered"`.
2. Drop the `scrip` label from per-order metrics, or cap it to a top-N set with an `"other"` bucket.
3. Record metrics after validation, or sanitise the label first (constant-time lookup in the registry; no allocation on the approve path).
4. When metrics move behind the injected `RiskObserver` (plan R1), enforce the same label policy in the observer contract so any adapter inherits it.

**Verification**: `tests/metrics_cardinality_tests.rs` feeding 10k bogus algo ids and asserting the series count stays bounded by a small constant; test that registered ids still get their own series; `cargo clippy --all-targets -- -D warnings`, `cargo test -p tantra-risk --profile prod`, `cargo bench` to confirm no SLA regression.

## Related

- Plan R1 (Prometheus global singleton and `f64` gauges move behind an observer port).
- [0008](0008-bounded-rate-windows.md) (unbounded tracker maps), [0012](0012-decision-exchange-and-algo-id-typing.md) (validated `AlgoId`).
