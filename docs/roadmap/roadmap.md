# `vyapaar-risk` — TDD & Engineering Roadmap

## Milestone 1: Core Risk Engine & SEBI Algo Registry (P0)
- [ ] Port `algo_registry.rs`, `state.rs`, `limits.rs`, `error.rs`, and `refusal.rs`.
- [ ] Port `otr.rs` (sliding-window OTR tracker) and `engine.rs` (`RiskEngine::check_order` and zero-allocation `check_order_fast`).

## Milestone 2: Dynamic Sizing Gate & Observability Metrics (P0)
- [ ] Port `sizing_gate.rs` (`validate_sizing_request`, `SizingRiskRefusal`).
- [ ] Port `metrics.rs` (`RiskMetrics`, `RISK_EVAL_BUCKETS`).

## Milestone 3: Property-Based & Latency Verification (P0)
- [ ] Port all integration and `proptest` suites into `tests/`.
- [ ] Verify zero `#[cfg(test)]` blocks inside `src/`.
