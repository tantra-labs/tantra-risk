# `tantra-risk` — Historical Migration Map (honba-mon → tantra-risk)

> **Tier**: Tier 1A (Depends only on `tantra-core`)
> **Source**: `honba-workspace/honba-mon`
> **Target**: `tantra-risk`
> **Status**: Migration complete (all modules ported).
> **Milestones & verification**: see [`implementation-record.md`](implementation-record.md).

## File-by-File Migration Map

| Source Path (`honba-mon/`) | Target Path (`tantra-risk/`) | Action & DDD Refactoring Required |
| :--- | :--- | :--- |
| `Cargo.toml` | `Cargo.toml` | Ensure `tantra-core = { path = "../tantra-core" }` and all dependencies (`rust_decimal`, `serde`, `chrono`, `thiserror`, `tracing`, `schemars`, `prometheus`) match. |
| `Makefile` | `Makefile` | Port `Makefile` and replace `honba-mon` with `tantra-risk`. |
| `src/lib.rs` | `src/lib.rs` | Port module declarations and public re-exports; update doc comments from `honba-mon` / `honba_core` to `tantra-risk` / `tantra_core`. |
| `src/algo_registry.rs` | `src/algo_registry.rs` | Replace `honba_core` with `tantra_core`. |
| `src/engine.rs` | `src/engine.rs` | Replace `honba_core` with `tantra_core`. Verify zero heap allocations on `check_order_fast`. |
| `src/error.rs` | `src/error.rs` | Replace `honba_core` with `tantra_core`. |
| `src/limits.rs` | `src/limits.rs` | Replace `honba_core` with `tantra_core`. |
| `src/metrics.rs` | `src/metrics.rs` | Port Prometheus histogram / counters / gauge (`RiskMetrics`, `RISK_EVAL_BUCKETS`, `GLOBAL_RISK_METRICS`). |
| `src/otr.rs` | `src/otr.rs` | Replace `honba_core` with `tantra_core`. |
| `src/refusal.rs` | `src/refusal.rs` | Replace `honba_core` with `tantra_core`. |
| `src/sizing_gate.rs` | `src/sizing_gate.rs` | Replace `honba_core` with `tantra_core`. |
| `src/state.rs` | `src/state.rs` | Replace `honba_core` with `tantra_core`. |
| `src/tool_hints.rs` | `src/tool_hints.rs` | New module: `ToolRiskHint`, `risk_classification` (MCP gateway integration). |
| `tests/*.rs` | `tests/*.rs` | Copy all test files from `honba-mon/tests/` into `tantra-risk/tests/`, replacing `honba_mon` → `tantra_risk` and `honba_core` → `tantra_core`. Extract any inline `#[cfg(test)]` from `src/*.rs` into `tests/`. |

## Test Inventory (post-migration)

| File | Coverage |
|------|----------|
| `tests/risk_tests.rs` | Kill switch, per-algo halt, price bands, market circuit, SEBI algo ID, OTR circuit breaker, order rate limiter, scrip OTR, freeze slicing, cooldown auto-resume. |
| `tests/otr_tests.rs` | Cumulative & sliding-window OTR, grace period, warning-zone boundaries, 5-min window eviction, cancellations/modifications, fills. |
| `tests/registry_tests.rs` | Register / suspend / reactivate / unregister. |
| `tests/phase7_tests.rs` | Sizing gate (7.1), Prometheus metrics + W3C `TraceContext` (7.2), freeze slices / concentration / MIS leverage / fast-check refusals (7.3). |
| `tests/proptest_tests.rs` | Property-based tests: randomized order streams, adversarial inputs, edge cases; `tests/proptest_tests.proptest-regressions` holds any recorded failing cases. |
| `tests/tool_hints_tests.rs` | `risk_classification` per `Decision` variant, `mcp_key()` vocabulary, serde round-trip. |

## Strict Invariants Checklist

- [x] **Zero-I/O**: No `tokio`, no network/file I/O, no `Utc::now()` / `SystemTime::now()` in `src/` (only `Instant::now()` for latency metrics).
- [x] **Zero Inline Tests**: `grep -rn "#\[cfg(test)\]" src/` returns 0 matches.
- [x] **Zero Floats**: All notional, price-band, loss, and OTR calculations use `rust_decimal::Decimal`.
- [x] **Latency SLO**: `p99.99 < 15µs` via zero-allocation `RiskEngine::check_order_fast`.

## Related

- [`implementation-record.md`](implementation-record.md) — completed milestones and verification status.
- [`../gaps/README.md`](../gaps/README.md) — open gaps discovered after the migration.
- [`../plans/README.md`](../plans/README.md) — plans index.
