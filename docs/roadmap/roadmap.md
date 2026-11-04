# `vyapaar-risk` — TDD & Engineering Roadmap

## Milestone 1: Core Risk Engine & SEBI Algo Registry (P0)
- [x] Port `algo_registry.rs`, `state.rs`, `limits.rs`, `error.rs`, and `refusal.rs`.
- [x] Port `otr.rs` (sliding-window OTR tracker) and `engine.rs` (`RiskEngine::check_order` and zero-allocation `check_order_fast`).

## Milestone 2: Dynamic Sizing Gate & Observability Metrics (P0)
- [x] Port `sizing_gate.rs` (`validate_sizing_request`, `SizingRiskRefusal`).
- [x] Port `metrics.rs` (`RiskMetrics`, `RISK_EVAL_BUCKETS`).

## Milestone 3: Property-Based & Latency Verification (P0)
- [x] Port all integration and `proptest` suites into `tests/`.
- [x] Verify zero `#[cfg(test)]` blocks inside `src/`.
