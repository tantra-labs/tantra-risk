# `vyapaar-risk` — Migration Verification Milestone

Verification commands for the `honba-mon → vyapaar-risk` migration:

```bash
cd /home/pmallapp/Devel/Finance/vyapaar-labs/vyapaar-risk
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Migration Milestone Status

| Milestone | Status |
|-----------|--------|
| `algo_registry.rs` — SEBI Algo ID registry | Complete |
| `state.rs` — `AlgoState`, `GlobalState`, `MarketCircuitHalt` | Complete |
| `limits.rs` — `RiskLimits`, `OrderRateLimit` | Complete |
| `error.rs` — `RiskError` | Complete |
| `refusal.rs` — `RiskRefusal` | Complete |
| `otr.rs` — `OtrTracker`, sliding-window OTR | Complete |
| `engine.rs` — `RiskEngine::check_order` / `check_order_fast` | Complete |
| `sizing_gate.rs` — `validate_sizing_request` | Complete |
| `metrics.rs` — `RiskMetrics`, `RISK_EVAL_BUCKETS` | Complete |
| `tool_hints.rs` — `ToolRiskHint`, `risk_classification` | Complete |
| `tests/` — all test files ported | Complete |
| Zero inline `#[cfg(test)]` in `src/` | Complete |
| `cargo fmt --check` | Clean |
| `cargo clippy --all-targets -- -D warnings` | Clean |
| `cargo test` | Passing |
