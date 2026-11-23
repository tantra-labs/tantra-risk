# 0008 — Unbounded growth: `order_history` never evicted without a rate limit; OTR window scans are O(n) per check

**Status**: open
**Filed**: 2026-11-23 — workspace design review (root plan 0021) · **Kind**: design-flaw · **Severity**: high
**Plan**: [0003](../plans/0003-design-review-2026-11.md)

## What's stubbed or hardcoded

- `src/engine.rs:806` and `~1140` (fast path): `self.order_history.push_back(now)` runs unconditionally, but eviction (`src/engine.rs:785-790`, `1127`) sits inside `if let Some(rate_limit)`. With `order_rate_limit = None` (the default) the deque grows one entry per approved order forever.
- `src/otr.rs:~125-150` `window_otr` makes two full `iter().filter().count()` passes over every event in the window on every call. The engine calls it for the algo tracker and the scrip tracker per order (`src/engine.rs:~606`, `~644`): cost is O(events in the minute) x 2, not O(1).
- `src/otr.rs:~84`: `events` has no capacity cap; `scrip_trackers` / `algo_trackers` maps are keyed by external input with no cap or daily reset.
- `benches/risk_benchmarks.rs:~24` uses a fixed `now`, so nothing ever expires and the bench measures neither the growing deque nor eviction.

## Why

At the 50k orders/s burst quoted in `src/metrics.rs`, the one-minute window holds about 3M events and each check scans them twice, so the 15 µs SLO is violated by orders of magnitude. Memory also grows without bound in a long-running process. This breaks `.agents/rules/scalability-and-concurrency.md` section 3 (bounded memory).

## How to close it

1. Keep running counters (placements, modifications, cancellations, fills) in `OtrTracker` and decrement them on eviction, giving O(1) amortised `window_otr`.
2. Push to `order_history` only when an `order_rate_limit` exists, and evict on every push.
3. Cap deque capacity and tracker maps; on overflow return a fail-closed `CapacityExceeded` refusal.
4. Add a session reset hook shared with [0009](0009-trading-day-rollover.md).
5. Re-baseline the benchmark with an advancing `now` and 100k events in the window; the sub-15 µs p99.99 target is the acceptance bar.

**Verification**: golden/proptest equivalence between the old and the counter-based `window_otr`; `tests/bounded_state_tests.rs` asserting deque and map sizes stay under cap over 1M simulated orders; Criterion bench with advancing `now`; `cargo clippy --all-targets -- -D warnings`, `cargo test -p tantra-risk --profile prod`.

## Related

- Root plan 0019 P3.2 covers only merging the two check pipelines, not this.
- [0009](0009-trading-day-rollover.md), [0013](0013-metrics-label-cardinality.md), [0015](0015-latency-sla-ci-gate.md).
