# `vyapaar-risk` — Historical Migration Map (honba-mon → vyapaar-risk)

> **Tier**: Tier 1A (Depends only on `vyapaar-core`)
> **Source**: `honba-workspace/honba-mon`
> **Target**: `vyapaar-risk`
> **Status**: Migration complete (all modules ported).

## File-by-File Migration Map

| Source Path (`honba-mon/`) | Target Path (`vyapaar-risk/`) | Action & DDD Refactoring Required |
| :--- | :--- | :--- |
| `Cargo.toml` | `Cargo.toml` | Ensure `vyapaar-core = { path = "../vyapaar-core" }` and all dependencies (`rust_decimal`, `serde`, `chrono`, `thiserror`, `tracing`, `schemars`) match. |
| `Makefile` | `Makefile` | Port `Makefile` and replace `honba-mon` with `vyapaar-risk`. |
| `src/lib.rs` | `src/lib.rs` | Port module declarations and public re-exports; update doc comments from `honba-mon` / `honba_core` to `vyapaar-risk` / `vyapaar_core`. |
| `src/algo_registry.rs` | `src/algo_registry.rs` | Replace `honba_core` with `vyapaar_core`. |
| `src/engine.rs` | `src/engine.rs` | Replace `honba_core` with `vyapaar_core`. Verify zero heap allocations on `check_order_fast`. |
| `src/error.rs` | `src/error.rs` | Replace `honba_core` with `vyapaar_core`. |
| `src/limits.rs` | `src/limits.rs` | Replace `honba_core` with `vyapaar_core`. |
| `src/metrics.rs` | `src/metrics.rs` | Port atomic Prometheus histogram and counters. |
| `src/otr.rs` | `src/otr.rs` | Replace `honba_core` with `vyapaar_core`. |
| `src/refusal.rs` | `src/refusal.rs` | Replace `honba_core` with `vyapaar_core`. |
| `src/sizing_gate.rs` | `src/sizing_gate.rs` | Replace `honba_core` with `vyapaar_core`. |
| `src/state.rs` | `src/state.rs` | Replace `honba_core` with `vyapaar_core`. |
| `src/tool_hints.rs` | `src/tool_hints.rs` | New module: `ToolRiskHint`, `risk_classification` (MCP gateway integration). |
| `tests/*.rs` | `tests/*.rs` | Copy all test files from `honba-mon/tests/` into `vyapaar-risk/tests/`, replacing `honba_mon` → `vyapaar_risk` and `honba_core` → `vyapaar_core`. Extract any inline `#[cfg(test)]` from `src/*.rs` into `tests/`. |

## Strict Invariants Checklist

- [x] **Zero-I/O**: No `tokio`, no network/file I/O, no `Utc::now()` in `src/`.
- [x] **Zero Inline Tests**: `grep -rn "#\[cfg(test)\]" src/` must return 0 matches.
- [x] **Zero Floats**: All notional, price-band, loss, and OTR calculations use `rust_decimal::Decimal`.
- [x] **Latency SLO**: `p99.99 < 15µs` via zero-allocation `RiskEngine::check_order_fast`.

## Verification Commands

```bash
# Run from the repository root
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Migration Execution Status

| Milestone | Status |
|-----------|--------|
| Module 1: Core Risk Engine & SEBI Algo Registry | Complete |
| Module 2: Dynamic Sizing Gate & Observability Metrics | Complete |
| Module 3: Property-Based & Latency Verification | Complete |
| `cargo fmt --check` | Clean |
| `cargo clippy --all-targets -- -D warnings` | Clean |
| `cargo test` | Passing |
