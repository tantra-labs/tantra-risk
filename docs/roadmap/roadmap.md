# `vyapaar-risk` — TDD & Engineering Roadmap

## Milestone 1: Core Risk Engine & SEBI Algo Registry (P0) — [COMPLETED]
- [x] Port `algo_registry.rs`, `state.rs`, `limits.rs`, `error.rs`, and `refusal.rs`.
- [x] Port `otr.rs` (sliding-window OTR tracker) and `engine.rs` (`RiskEngine::check_order` and zero-allocation `check_order_fast`).

## Milestone 2: Dynamic Sizing Gate & Observability Metrics (P0) — [COMPLETED]
- [x] Port `sizing_gate.rs` (`validate_sizing_request`, `SizingRiskRefusal`).
- [x] Port `metrics.rs` (`RiskMetrics`, `RISK_EVAL_BUCKETS`).

## Milestone 3: Property-Based & Latency Verification (P0) — [COMPLETED]
- [x] Port all integration and `proptest` suites into `tests/`.
- [x] Verify zero `#[cfg(test)]` blocks inside `src/`.

## Migration Verification — [COMPLETED]
- [x] `cargo fmt --check` — clean
- [x] `cargo clippy --all-targets -- -D warnings` — clean
- [x] `cargo test` — passing
- [x] Zero-I/O domain purity — no `tokio`, no network/file I/O, no `Utc::now()` in `src/`
- [x] Zero floats — all notional, price-band, loss, OTR in `rust_decimal::Decimal`
- [x] Latency SLO — `p99.99 < 15µs` via `check_order_fast`

See `docs/roadmap/migration-verification.md` for full migration milestone status.
